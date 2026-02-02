use std::sync::Arc;

use actix_web::{App, HttpServer, web};
use bincode::{config::standard, encode_to_vec};
use rusqlite::Connection;
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::Mutex,
};

use crate::{
    dht::{
        find_node::find_node,
        routing::{FindNodeResult, RoutingTable},
    },
    protocol::opcode::{UdpResponse, handle_udp_request},
    storage::db::init_db,
};

async fn start_tcp_server(routing_table: Arc<Mutex<RoutingTable>>) -> std::io::Result<()> {
    let routing_table = web::Data::new(routing_table.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(routing_table.clone())
            .route("/node/{id}", web::get().to(find_node))
    })
    .bind(("127.0.0.1", 1234))?
    .run()
    .await
}

async fn start_udp_server(conn: Arc<Mutex<Connection>>, routing_table: Arc<Mutex<RoutingTable>>) {
    let udp_socket = UdpSocket::bind("127.0.0.1:1235").await.unwrap();
    let port = udp_socket.local_addr().unwrap().port();
    println!("UDP server running on port {}", port);

    let mut buf = [0; 1024];
    loop {
        match udp_socket.recv_from(&mut buf).await {
            Ok((size, src)) => {
                let response = handle_udp_request(&buf, conn.clone(), size, src, routing_table.clone()).await;
                let bytes = bincode::encode_to_vec(response, standard()).unwrap();
                udp_socket.send_to(&bytes, src).await.unwrap();
            }
            Err(e) => {
                eprintln!("Error receiving UDP packet: {}", e);
                break;
            }
        }
    }
}

pub async fn start_server() {
    let conn = Arc::new(Mutex::new(init_db().unwrap()));
    let routing_table = Arc::new(Mutex::new(RoutingTable::new()));

    let udp_rt = routing_table.clone();

    tokio::spawn(async move {
        start_udp_server(conn,udp_rt).await;
    });

    let tcp_rt = routing_table.clone();
    if let Err(e) = start_tcp_server(tcp_rt).await {
        eprintln!("TCP server error: {}", e);
    }
}
