use std::{
    net::{SocketAddr},
    sync::Arc,
};

use crate::{
    dht::{node_lookup::node_lookup, routing::RoutingTable},
    protocol::opcode::{UdpMessage, UdpRequest, UdpResponse},
    startup::RpcManager,
    utils::{constant::ID_BYTES, generate_id::generate_id, node::Identity},
};
use bincode::config::standard;
use futures::future::join_all;
use rusqlite::Connection;
use tokio::{net::UdpSocket, sync::Mutex};

pub async fn store_value(
    key: [u8; ID_BYTES],
    value: Vec<u8>,
    routing_table: Arc<Mutex<RoutingTable>>,
    rpc_manager: Arc<Mutex<RpcManager>>,
    socket: Arc<UdpSocket>,
) {
    let destination_nodes = node_lookup(key, routing_table, rpc_manager, socket.clone()).await;
    let sender_id = Identity::load().unwrap().node_id;
    let request_id = generate_id();
    let request = UdpMessage::Request(UdpRequest::Store {
        request_id,
        sender_id,
        key,
        value,
    });
    let bytes = bincode::encode_to_vec(&request, standard()).unwrap();
    let socket = Arc::new(socket.clone());
    let futures = destination_nodes.into_iter().map(|node| {
        let bytes = bytes.clone();
        let socket = Arc::clone(&socket);
        async move {
            let addr: SocketAddr = format!("{}:{}", node.ip, node.port).parse().unwrap();
            socket.send_to(&bytes, &addr).await
        }
    });
    let responses = join_all(futures).await;
}

pub fn store_locally(
    conn: &Connection,
    request_id: [u8; ID_BYTES],
    node_id: [u8; ID_BYTES],
    value: Vec<u8>,
) -> Result<UdpResponse, Box<dyn std::error::Error>> {
    conn.execute(
        "INSERT INTO dht_storage (key, value) VALUES (?1, ?2)",
        (node_id.to_vec(), value),
    )?;
    Ok(UdpResponse::Store { request_id })
}
