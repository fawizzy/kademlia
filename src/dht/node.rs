use std::{fs, net::IpAddr};

use crate::utils::{
    constant::{ID_BYTES, PORT},
    crypto::generate_id,
    distance::xor_distance,
};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct Config {
    node_id: [u8; 20],
    // ip: String,
    port: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Encode, Decode, Copy)]
pub struct Node {
    pub ip: IpAddr,
    pub port: u16,

    #[serde(serialize_with = "serialize_node_id_as_hex")]
    pub node_id: [u8; 20],
}

#[derive(Debug, Clone)]
pub struct Identity {
    pub port: String,
    pub node_id: [u8; 20],
}

impl Identity {
    pub fn new() -> Self {
        let node_id = generate_id();
        let contents = json!({
            "node_id": node_id,
            "port": PORT
        });
        let serialized = serde_json::to_string(&contents);
        let _ = fs::write("config.json", serialized.unwrap());
        Identity {
            node_id,
            port: PORT.to_string(),
        }
    }

    pub fn load() -> Result<Identity, Box<dyn std::error::Error>> {
        let config = fs::read_to_string("config.json")?;

        let value: Config = serde_json::from_str(&config)?;
        return Ok(Identity {
            node_id: value.node_id,
            port: value.port,
        });
    }

    pub fn init() -> Self {
        let node = match Identity::load() {
            Ok(node) => node,
            Err(_) => Identity::new(),
        };

        node
    }

    pub fn distance_to_self(node_id: [u8; ID_BYTES]) -> [u8; ID_BYTES] {
        let id = Identity::load().unwrap().node_id;
        xor_distance(id, node_id)
    }
}

fn serialize_node_id_as_hex<S>(bytes: &[u8; 20], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&hex::encode(bytes))
}
