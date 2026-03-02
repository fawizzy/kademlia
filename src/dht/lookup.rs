use std::{net::SocketAddr, sync::Arc};

use ::futures::future::join_all;
use tokio::{net::UdpSocket, sync::Mutex};

use crate::{
    dht::node::Node,
    dht::routing::{FindNodeResult, RoutingTable},
    network::{
        client::{find_node_request, find_value_request},
        messages::{FindValueResult, UdpResponse},
        rpc::RpcManager,
    },
    utils::{constant::K, distance::xor_distance},
};

pub async fn node_lookup(
    target_id: [u8; 20],
    routing_table: Arc<Mutex<RoutingTable>>,
    rpc_manager: Arc<Mutex<RpcManager>>,
    socket: Arc<UdpSocket>,
) -> Vec<Node> {
    const ALPHA: usize = 3;
    let mut shortlist = {
        let rt = routing_table.lock().await;
        match rt.find_node(target_id) {
            FindNodeResult::Closest(nodes) => nodes,
            FindNodeResult::Found(node) => vec![node],
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

            find_node_request(
                addr,
                target_id,
                routing_table.clone(),
                rpc_manager.clone(),
                socket.clone(),
            )
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

pub async fn value_lookup(
    key: [u8; 20],
    routing_table: Arc<Mutex<RoutingTable>>,
    rpc_manager: Arc<Mutex<RpcManager>>,
    socket: Arc<UdpSocket>,
) -> Option<Vec<u8>> {
    const ALPHA: usize = 3;
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
