use std::{net::IpAddr, sync::Arc};

use actix_web::{HttpResponse, Responder, web};
use hex::decode;
use tokio::sync::Mutex;

use crate::{
    dht::routing::{FindNodeResult, RoutingTable},
    utils::node,
};

pub async fn find_node(
    path: web::Path<String>,
    routing_table: web::Data<Arc<Mutex<RoutingTable>>>,
) -> impl Responder {
    let node_id = decode(path.into_inner()).unwrap();

    let node_id: [u8; 20] = node_id
        .try_into()
        .expect("node_id must be exactly 20 bytes");

    let mut node_results = match routing_table.lock().await.find_node(node_id) {
        FindNodeResult::Closest(nodes) => nodes,
        FindNodeResult::Found(node) => vec![node],
    };

    node_results.push(node::Node {
        ip: IpAddr::V4("127.0.0.1".parse().unwrap()),
        port: 0000,
        node_id,
    });
    HttpResponse::Found().json(node_results)
}
