use std::net::IpAddr;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::{
    dht::bucket::Bucket,
    utils::{
        constant::{ID_BYTES, K},
        distance::{compare_distance, xor_distance},
        node::{Identity, Node},
    },
};
#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum FindNodeResult {
    Found(Node),
    Closest(Vec<Node>),
}

#[derive(Debug)]
pub struct RoutingTable {
    pub buckets: Vec<Bucket>,
}

impl RoutingTable {
    pub fn new() -> Self {
        let buckets = vec![Bucket::new(); ID_BYTES * 8];

        RoutingTable { buckets }
    }

    pub fn insert_node(&mut self, node: &Node) {
        let distance = Identity::distance_to_self(node.node_id);
        let bucket_index = Bucket::get_bucket_index(distance);

        let bucket = &mut self.buckets[bucket_index];

        if let Some(pos) = bucket.nodes.iter().position(|n| {
            n.clone()
                .unwrap_or(Node {
                    ip: IpAddr::V4("127.0.0.1".parse().unwrap()),
                    port: 0000,
                    node_id: [0; 20],
                })
                .node_id
                == node.node_id
        }) {
            let n = bucket.nodes.remove(pos).unwrap();
            bucket.nodes.push_back(n);
            return;
        }

        if bucket.nodes.len() < K as usize {
            bucket.nodes.push_back(Some(node.clone()));
            return;
        }
    }

    pub fn get_all_nodes(&self) -> Vec<Node> {
        let mut all_nodes = vec![];

        for bucket in &self.buckets {
            for node_opt in &bucket.nodes {
                if let Some(node) = node_opt {
                    all_nodes.push(node.clone());
                }
            }
        }

        let local_node = Identity::load();

        all_nodes
    }

    pub fn find_node(&self, node_id: [u8; ID_BYTES]) -> FindNodeResult {
        let distance = Identity::distance_to_self(node_id);
        let index = Bucket::get_bucket_index(distance);
        let bucket = &self.buckets[index];

        for node in bucket.nodes.iter() {
            if let Some(n) = node {
                if n.node_id == node_id {
                    return FindNodeResult::Found(n.clone());
                }
            }
        }

        self.find_k_closest_node(node_id)
    }

    pub fn find_k_closest_node(&self, node_id: [u8; ID_BYTES]) -> FindNodeResult {
        let mut all_nodes = self.get_all_nodes();
        all_nodes.sort_by(|a, b| {
            let distance_a = xor_distance(a.node_id, node_id);
            let distance_b = xor_distance(b.node_id, node_id);
            compare_distance(distance_a, distance_b)
        });

        FindNodeResult::Closest(all_nodes.iter().map(|n| n.clone()).collect::<Vec<Node>>())
    }

}
