use std::collections::HashMap;
use tokio::sync::oneshot;

use crate::network::messages::UdpResponse;
use crate::utils::constant::ID_BYTES;

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
