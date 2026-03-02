use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::dht::routing::FindNodeResult;

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum FindValueResult {
    Found(Vec<u8>),
    Closest(FindNodeResult),
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum UdpMessage {
    Request(UdpRequest),
    Response(UdpResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum UdpResponse {
    PONG {
        request_id: [u8; 20],
    },
    FindNodeResult {
        result: FindNodeResult,
        request_id: [u8; 20],
    },
    Store {
        request_id: [u8; 20],
    },
    FindValue {
        result: FindValueResult,
        request_id: [u8; 20],
    },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Encode, Decode)]
pub enum UdpRequest {
    Ping {
        request_id: [u8; 20],
        sender_id: [u8; 20],
    },
    FindNode {
        request_id: [u8; 20],
        sender_id: [u8; 20],
        target_id: [u8; 20],
    },
    Store {
        request_id: [u8; 20],
        sender_id: [u8; 20],
        key: [u8; 20],
        value: Vec<u8>,
    },
    FindValue {
        request_id: [u8; 20],
        sender_id: [u8; 20],
        key: [u8; 20],
    },
}
