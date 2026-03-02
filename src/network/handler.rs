use rusqlite::Connection;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use crate::{
    dht::{node::Node, routing::RoutingTable},
    network::{
        messages::{FindValueResult, UdpRequest, UdpResponse},
        rpc::RpcManager,
    },
    storage::sqlite::{find_value, store_locally},
};

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
