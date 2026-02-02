use sha1::{Digest, Sha1};
use rand::Rng;

use crate::utils::constant::ID_BYTES;
pub fn generate_id() -> [u8; ID_BYTES] {
    let mut hasher = Sha1::new();
    let mut rng = rand::rng();
    let random: [u8; ID_BYTES] = rng.random();
    hasher.update(&random);
    hasher.finalize().into()
}