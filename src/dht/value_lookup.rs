use std::{
    net::{SocketAddr},
    sync::Arc,
    time::Duration,
};

use bincode::config::standard;
use futures::future::join_all;
use tokio::{
    net::UdpSocket, sync::{Mutex, oneshot}, time::timeout
};

use crate::{
    dht::routing::{FindNodeResult, RoutingTable},
    protocol::opcode::{FindValueResult, UdpMessage, UdpRequest, UdpResponse},
    startup::RpcManager,
    utils::{distance::xor_distance, generate_id::generate_id, node::Identity},
};

pub async fn value_lookup(
    key: [u8; 20],
    routing_table: Arc<Mutex<RoutingTable>>,
    rpc_manager: Arc<Mutex<RpcManager>>,
    socket: Arc<UdpSocket>,
) -> Option<Vec<u8>> {
    const ALPHA: usize = 3;
    const K: usize = 20;

    let mut shortlist = {
        let rt = routing_table.lock().await;
        match rt.find_node(key) {
            FindNodeResult::Closest(nodes) => nodes,
            FindNodeResult::Found(node) => vec![node],
        }
    };

    let mut queried = std::collections::HashSet::new();

    loop {
        shortlist.sort_by_key(|node| xor_distance(node.node_id, key));

        let to_query: Vec<_> = shortlist
            .iter()
            .filter(|node| !queried.contains(&node.node_id))
            .take(ALPHA)
            .cloned()
            .collect();

        if to_query.is_empty() {
            break;
        }

        for node in &to_query {
            queried.insert(node.node_id);
        }

        let futures = to_query.into_iter().map(|node| {
            let addr: SocketAddr = format!("{}:{}", node.ip, node.port).parse().unwrap();

            find_value_request(
                addr,
                key,
                routing_table.clone(),
                rpc_manager.clone(),
                socket.clone(),
            )
        });

        let responses = join_all(futures).await;

        let mut found_closer = false;

        for response in responses {
            match response {
                UdpResponse::FindValue { result, .. } => {
                    if let FindValueResult::Found(value) = result {
                        return Some(value);
                    }

                    if let FindValueResult::Closest(FindNodeResult::Closest(nodes)) = result {
                        for node in nodes {
                            if !shortlist.iter().any(|n| n.node_id == node.node_id) {
                                shortlist.push(node);
                                found_closer = true;
                            }
                        }
                    }
                }

                _ => {}
            }
        }

        shortlist.sort_by_key(|node| xor_distance(node.node_id, key));
        shortlist.truncate(K);

        if !found_closer {
            break;
        }
    }

    None
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
