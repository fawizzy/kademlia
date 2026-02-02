use bincode::{Decode, Encode, config::standard};
use hex::{decode, encode};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::{convert::TryInto, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    dht::{
        routing::{FindNodeResult, RoutingTable},
        store::store_locally,
    },
    utils::node::Node,
};

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum UdpResponse {
    PONG,
    FindNodeResult(FindNodeResult),
    Store,
    None,
}

#[derive(Debug, Encode, Decode)]
pub enum UdpRequest {
    Ping {
        sender_id: [u8; 20],
    },
    FindNode {
        sender_id: [u8; 20],
        target_id: [u8; 20],
    },
    Store {
        key: [u8; 20],
        value: Vec<u8>,
    },
}

pub async fn handle_udp_request(
    req: &[u8; 1024],
    conn: Arc<Mutex<Connection>>,
    size: usize,
    src: SocketAddr,
    routing_table: Arc<Mutex<RoutingTable>>,
) -> UdpResponse {
    let (request, _): (UdpRequest, _) =
        bincode::decode_from_slice(&req[..size], standard()).unwrap();

    match request {
        UdpRequest::Ping { sender_id } => {
            let requesting_node = Node {
                ip: src.ip(),
                port: src.port(),
                node_id: sender_id,
            };
            routing_table.lock().await.insert_node(&requesting_node);
            return UdpResponse::PONG;
        }
        UdpRequest::FindNode {
            sender_id,
            target_id,
        } => {
            let rt = routing_table.lock().await;
            let nodes = rt.find_node(target_id);

            return UdpResponse::FindNodeResult(nodes);
        }

        UdpRequest::Store { key, value } => {
            let conn = conn.lock().await;
            store_locally(&conn, key, value).unwrap()
        }

        _ => {
            // Handle other message types
            return UdpResponse::None;
        }
    }
}
