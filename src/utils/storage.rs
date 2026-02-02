use crate::utils::constant::ID_BYTES;
use rusqlite::Connection;

pub fn store_locally(
    conn: &Connection,
    node_id: [u8; ID_BYTES],
    value: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute(
        "INSERT INTO dht_storage (key, value) VALUES (?1, ?2)",
        (node_id.to_vec(), value),
    )?;
    Ok(())
}

