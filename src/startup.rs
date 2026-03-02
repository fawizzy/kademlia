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
        find_node::find_node, node_lookup::node_lookup, routing::{FindNodeResult, RoutingTable}
    },
    protocol::opcode::{UdpMessage, UdpRequest, UdpResponse, process_udp_request},
    storage::db::init_db,
    utils::constant::ID_BYTES,
};

use std::collections::HashMap;
use tokio::sync::oneshot;


pub struct RpcManager {
    pending: HashMap<[u8; ID_BYTES], oneshot::Sender<UdpResponse>>,
}

impl RpcManager {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    pub fn insert(&mut self, request_id: [u8; ID_BYTES], tx: oneshot::Sender<UdpResponse>) {
        self.pending.insert(request_id, tx);
    }

    pub fn resolve(&mut self, request_id: [u8; ID_BYTES], response: UdpResponse) {
        if let Some(tx) = self.pending.remove(&request_id) {
            let _ = tx.send(response);
        }
    }
}

async fn start_tcp_server(
    udp_socket: Arc<UdpSocket>,
    // rpc_manager: Arc<Mutex<RpcManager>>
    routing_table: Arc<Mutex<RoutingTable>>,
) -> std::io::Result<()> {
   
    let routing_table = web::Data::new(routing_table.clone());
    let udp_socket = web::Data::new(udp_socket.clone());
    
    HttpServer::new(move || {
        App::new()
            .app_data(routing_table.clone())
            .app_data(udp_socket.clone())
            .route("/node/{id}", web::get().to(find_node))
    })
    .bind(("127.0.0.1", 1234))?
    .run()
    .await
}

async fn start_udp_server(
    udp_socket: Arc<UdpSocket>,
    conn: Arc<Mutex<Connection>>,
    routing_table: Arc<Mutex<RoutingTable>>,
    rpc_manager: Arc<Mutex<RpcManager>>,
) {
    let port = udp_socket.local_addr().unwrap().port();
    println!("UDP server running on port {}", port);

    let mut buf = [0; 1024];
    loop {
        match udp_socket.recv_from(&mut buf).await {
            Ok((size, src)) => {
                if let Ok((message, _)) =
                    bincode::decode_from_slice::<UdpMessage, _>(&buf[..size], standard())
                {
                    match message {
                        UdpMessage::Request(req) => {
                            let response = process_udp_request(
                                req,
                                conn.clone(),
                                src,
                                routing_table.clone(),
                                rpc_manager.clone(),
                                udp_socket.clone(),
                            )
                            .await;
                            let msg = UdpMessage::Response(response);
                            let bytes = bincode::encode_to_vec(msg, standard()).unwrap();
                            udp_socket.send_to(&bytes, src).await.unwrap();
                        }
                        UdpMessage::Response(res) => match res {
                            UdpResponse::PONG { request_id } => {
                                rpc_manager.lock().await.resolve(request_id, res)
                            }
                            UdpResponse::FindNodeResult { request_id, .. } => {
                                rpc_manager.lock().await.resolve(request_id, res)
                            }
                            UdpResponse::Store { request_id } => {
                                rpc_manager.lock().await.resolve(request_id, res)
                            }
                            UdpResponse::FindValue { request_id, .. } => {
                                rpc_manager.lock().await.resolve(request_id, res)
                            }
                            UdpResponse::None => {}
                        },
                    }
                } else {
                    eprintln!("Failed to decode UDP message from {}", src);
                }
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
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:1235").await.unwrap());
    let rpc_manager = Arc::new(Mutex::new(RpcManager::new()));

    let udp_rt = routing_table.clone();
    let udp_socket = socket.clone();
    let udp_rpc_manager = rpc_manager.clone();
    tokio::spawn(async move {
        start_udp_server(udp_socket, conn, udp_rt, udp_rpc_manager).await;
    });

    


    let tcp_rt = routing_table.clone();
    let tcp_udp_scocket = socket.clone();
    // let tcp_rpc_manager = rpc_manager.clone();

    if let Err(e) = start_tcp_server(tcp_udp_scocket, tcp_rt).await {
        eprintln!("TCP server error: {}", e);
    }
}
