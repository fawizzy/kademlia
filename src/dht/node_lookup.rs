use std::{
    net::{SocketAddr},
    sync::Arc,
    time::Duration,
};

use ::futures::future::join_all;
use bincode::config::standard;
use tokio::{net::UdpSocket, sync::{Mutex, oneshot}, time::timeout};

use crate::{
    dht::routing::{self, FindNodeResult, RoutingTable}, protocol::opcode::{UdpMessage, UdpRequest, UdpResponse}, startup::RpcManager, utils::{constant::K, distance::xor_distance, generate_id::generate_id, node::{Identity, Node}}
};

pub async fn node_lookup(target_id: [u8; 20], routing_table: Arc<Mutex<RoutingTable>>, rpc_manager: Arc<Mutex<RpcManager>>, socket: Arc<UdpSocket>,) -> Vec<Node>{
    const ALPHA: usize = 3;
    let mut shortlist = {
        let rt = routing_table.lock().await;
        match rt.find_node(target_id) {
            routing::FindNodeResult::Closest(nodes) => nodes,
            routing::FindNodeResult::Found(node) => vec![node],
        }
    };

    let mut queried = std::collections::HashSet::new();
    loop {
        // Sort by XOR distance
        shortlist.sort_by_key(|node| xor_distance(node.node_id, target_id));

        // Take α unqueried closest nodes
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

            find_node_request(addr, target_id, routing_table.clone(), rpc_manager.clone(), socket.clone())
        });

        let responses = join_all(futures).await;

        let mut found_closer = false;

        for response in responses {
            if let UdpResponse::FindNodeResult { result, .. } = response {
                if let FindNodeResult::Found(node) = result {
                    if !shortlist.iter().any(|n| n.node_id == node.node_id) {
                        shortlist.push(node);
                        found_closer = true;
                    }
                }

                if let FindNodeResult::Closest(nodes) = result {
                    for node in nodes {
                        if !shortlist.iter().any(|n| n.node_id == node.node_id) {
                            shortlist.push(node);
                            found_closer = true;
                        }
                    }
                }
            }
        }

        // Trim to K nodes
        shortlist.sort_by_key(|node| xor_distance(node.node_id, target_id));
        shortlist.truncate(K);

        if !found_closer {
            break;
        }
    }

    println!("Final shortlist: {:?}", shortlist);
    shortlist
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
    if let routing::FindNodeResult::Found(_) = nodes {
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
