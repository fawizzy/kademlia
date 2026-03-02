use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

use bincode::config::standard;
use tokio::sync::oneshot;
use tokio::time::timeout;

use crate::protocol::opcode::{UdpMessage, UdpRequest, UdpResponse};
use crate::startup::RpcManager;
use crate::utils::generate_id::generate_id;
use crate::utils::node::Node;

pub async fn ping_request(
    node: &Node,
    udp_socket: Arc<UdpSocket>,
    rpc_manager: Arc<Mutex<RpcManager>>,
) -> UdpResponse {
    let request_id = generate_id();
    let request = UdpRequest::Ping {
        request_id,
        sender_id: node.node_id,
    };
    let message = UdpMessage::Request(request);

    let bytes = bincode::encode_to_vec(&message, standard()).unwrap();

    let addr: SocketAddr = format!("{}:{}", node.ip, node.port).parse().unwrap();

    let socket = udp_socket;

    // Send ping
    socket.send_to(&bytes, addr).await.unwrap();
    println!("sending to {}", addr);
    let (tx, rx) = oneshot::channel();

    rpc_manager.lock().await.insert(request_id, tx);

    // Wait for response with timeout
    match timeout(Duration::from_secs(5), rx).await {
        Ok(Ok(response)) => {
            println!("{:?}", response);
            response
        }
        _ => UdpResponse::None,
    }
}
