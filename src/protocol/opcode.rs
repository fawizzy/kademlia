use bincode::{Decode, Encode, config::standard};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use crate::{
    dht::{
        find_value::find_value, node_lookup::node_lookup, routing::{FindNodeResult, RoutingTable}, store::store_locally
    },
    startup::RpcManager,
    utils::node::Node,
};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum FindValueResult {
    Found(Vec<u8>),
    Closest(FindNodeResult),
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum UdpMessage {
    Request(UdpRequest),
    Response(UdpResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum UdpResponse {
    PONG {
        request_id: [u8; 20],
    },
    FindNodeResult {
        result: FindNodeResult,
        request_id: [u8; 20],
    },
    Store {
        request_id: [u8; 20],
    },
    FindValue {
        result: FindValueResult,
        request_id: [u8; 20],
    },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum UdpRequest {
    Ping {
        request_id: [u8; 20],
        sender_id: [u8; 20],
    },
    FindNode {
        request_id: [u8; 20],
        sender_id: [u8; 20],
        target_id: [u8; 20],
    },
    Store {
        request_id: [u8; 20],
        sender_id: [u8; 20],
        key: [u8; 20],
        value: Vec<u8>,
    },
    FindValue {
        request_id: [u8; 20],
        sender_id: [u8; 20],
        key: [u8; 20],
    },
}

pub async fn process_udp_request(
    request: UdpRequest,
    conn: Arc<Mutex<Connection>>,
    src: SocketAddr,
    routing_table: Arc<Mutex<RoutingTable>>,
    rpc_manager: Arc<Mutex<RpcManager>>,
    udp_socket: Arc<UdpSocket>,
) -> UdpResponse {
    match request {
        UdpRequest::Ping {
            request_id,
            sender_id,
        } => {
            //  node_lookup([1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2], routing_table.clone()).await;
            let requesting_node = Node {
                ip: src.ip(),
                port: src.port(),
                node_id: sender_id,
            };
            routing_table
                .lock()
                .await
                .insert_node(&requesting_node, rpc_manager, udp_socket)
                .await;
            return UdpResponse::PONG { request_id };
        }
        UdpRequest::FindNode {
            request_id,
            sender_id,
            target_id,
        } => {
            let mut rt = routing_table.lock().await;
            let requesting_node = Node {
                ip: src.ip(),
                port: src.port(),
                node_id: sender_id,
            };
            rt.insert_node(&requesting_node, rpc_manager, udp_socket)
                .await;
            let nodes = rt.find_node(target_id);

            return UdpResponse::FindNodeResult {
                result: nodes,
                request_id,
            };
        }

        UdpRequest::Store {
            request_id,
            sender_id,
            key,
            value,
        } => {
            let conn = conn.lock().await;
            let mut rt = routing_table.lock().await;
            let requesting_node = Node {
                ip: src.ip(),
                port: src.port(),
                node_id: sender_id,
            };
            rt.insert_node(&requesting_node, rpc_manager, udp_socket)
                .await;
            store_locally(&conn, request_id, key, value).unwrap();

            UdpResponse::Store { request_id }
        }

        UdpRequest::FindValue {
            request_id,
            sender_id,
            key,
        } => {
            let conn = conn.lock().await;
            let mut rt = routing_table.lock().await;
            let requesting_node = Node {
                ip: src.ip(),
                port: src.port(),
                node_id: sender_id,
            };
            rt.insert_node(&requesting_node, rpc_manager, udp_socket)
                .await;

            if let Ok(Some(value)) = find_value(&conn, key) {
                return UdpResponse::FindValue {
                    result: FindValueResult::Found(value),
                    request_id,
                };
            }

            let nodes = rt.find_node(key);

            UdpResponse::FindValue {
                result: FindValueResult::Closest(nodes),
                request_id,
            }
        }
    }
}
