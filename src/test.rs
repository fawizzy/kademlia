use std::net::{IpAddr, SocketAddr};
use tokio::net::UdpSocket;
use bincode::{Decode, Encode, config::standard};
use std::error::Error;

// Make sure these match your server structs
#[derive(Debug, Clone, Encode, Decode)]
pub struct Node {
    pub ip: IpAddr,
    pub port: u16,
    pub node_id: [u8; 20],
}

#[derive(Debug, Encode, Decode)]
pub enum FindNodeRequest {
    FindNode { target_id: [u8; 20] },
}

#[derive(Debug, Encode, Decode)]
pub enum FindNodeResult {
    Closest(Vec<Node>),
    Found(Node),
}

#[derive(Debug, Encode, Decode)]
pub enum UdpResponse {
    Pong,
    FindNodeResult(FindNodeResult),
    // add other variants if needed
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Bind a local UDP socket (any free port)
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let server_addr: SocketAddr = "127.0.0.1:1235".parse()?; // your server

    // Build a sample request (replace with your own target_id)
    let request = FindNodeRequest::FindNode {
        target_id: [0u8; 20],
    };

    // Encode the request to bytes
    let bytes = bincode::encode_to_vec(&request, standard())?;

    // Send the request to the server
    socket.send_to(&bytes, &server_addr).await?;
    println!("Sent FIND_NODE request to {}", server_addr);

    // Prepare buffer to receive response
    let mut buf = [0u8; 1024];
    let (size, src) = socket.recv_from(&mut buf).await?;
    println!("Received {} bytes from {}", size, src);

    // Decode the response
    let (response, _): (UdpResponse, _) = bincode::decode_from_slice(&buf[..size], standard())?;
    println!("Decoded response: {:#?}", response);

    // If it’s a list of nodes, print each node nicely
    if let UdpResponse::FindNodeResult(FindNodeResult::Closest(nodes)) = response {
        for node in nodes {
            println!(
                "Node: {}:{} ID={:x?}",
                node.ip,
                node.port,
                node.node_id
            );
        }
    }

    Ok(())
}
