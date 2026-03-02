use std::{net::SocketAddr, sync::Arc, time::Duration};

use bincode::config::standard;
use futures::future::join_all;
use tokio::{
    net::UdpSocket,
    sync::{Mutex, oneshot},
    time::timeout,
};

use crate::utils::constant::ID_BYTES;
use crate::{
    dht::{
        lookup::node_lookup,
        node::{Identity, Node},
        routing::{FindNodeResult, RoutingTable},
    },
    network::{
        messages::{UdpMessage, UdpRequest, UdpResponse},
        rpc::RpcManager,
    },
    utils::crypto::generate_id,
};

pub async fn store_value(
    key: [u8; ID_BYTES],
    value: Vec<u8>,
    routing_table: Arc<Mutex<RoutingTable>>,
    rpc_manager: Arc<Mutex<RpcManager>>,
    socket: Arc<UdpSocket>,
) {
    let destination_nodes =
        node_lookup(key, routing_table, rpc_manager.clone(), socket.clone()).await;
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
    let _responses = join_all(futures).await;
}

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

pub async fn find_node_request(
    server_addr: SocketAddr,
    target_id: [u8; 20],
    routing_table: Arc<Mutex<RoutingTable>>,
    rpc_manager: Arc<Mutex<RpcManager>>,
    socket: Arc<UdpSocket>,
) -> UdpResponse {
    let sender_id = Identity::load().unwrap().node_id;

    let rt = routing_table.lock().await;
    let nodes = rt.find_node(target_id);

    let request_id = generate_id();
    if let FindNodeResult::Found(_) = nodes {
        return UdpResponse::FindNodeResult {
            result: nodes,
            request_id,
        };
    }

    let request = UdpRequest::FindNode {
        request_id,
        sender_id,
        target_id,
    };

    let message = UdpMessage::Request(request);
    let bytes = bincode::encode_to_vec(&message, standard()).unwrap();
    socket.send_to(&bytes, &server_addr).await.unwrap();

    println!("Sent FIND_NODE {:?} request to  {}", target_id, server_addr);

    let (tx, rx) = oneshot::channel();

    rpc_manager.lock().await.insert(request_id, tx);

    match timeout(Duration::from_secs(5), rx).await {
        Ok(Ok(response)) => {
            println!("{:?}", response);
            return response;
        }
        _ => return UdpResponse::None,
    }
}

pub async fn find_value_request(
    server_addr: SocketAddr,
    key: [u8; 20],
    routing_table: Arc<Mutex<RoutingTable>>,
    rpc_manager: Arc<Mutex<RpcManager>>,
    socket: Arc<UdpSocket>,
) -> UdpResponse {
    let sender_id = Identity::load().unwrap().node_id;

    let rt = routing_table.lock().await;
    let nodes = rt.find_node(key);

    let request_id = generate_id();
    if let FindNodeResult::Found(_) = nodes {
        return UdpResponse::FindNodeResult {
            result: nodes,
            request_id,
        };
    }

    let request = UdpRequest::FindValue {
        request_id,
        sender_id,
        key,
    };

    let message = UdpMessage::Request(request);
    let bytes = bincode::encode_to_vec(&message, standard()).unwrap();
    socket.send_to(&bytes, &server_addr).await.unwrap();
    println!("Sent FIND_VALUE {:?} request to  {}", key, server_addr);

    let (tx, rx) = oneshot::channel();

    rpc_manager.lock().await.insert(request_id, tx);

    match timeout(Duration::from_secs(5), rx).await {
        Ok(Ok(response)) => {
            println!("{:?}", response);
            return response;
        }
        _ => return UdpResponse::None,
    }
}
