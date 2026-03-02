use crate::network::messages::UdpResponse;
use crate::utils::constant::ID_BYTES;
use rusqlite::Connection;

pub fn init_db() -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open("kademlia.db")?;
    println!("sqlite database connected.....");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dht_storage (
                key BLOB PRIMARY KEY,
                value BLOB NOT NULL
            )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS node (
                key BLOB PRIMARY KEY,
                ip TEXT NOT NULL,
                port TEXT NOT NULL,
                last_seen INTEGER NOT NULL,
                bucket_index INTEGER NOT NULL,
                stored_at TEXT NOT NULL
            )",
        (),
    )?;

    Ok(conn)
}

pub fn store_locally(
    conn: &Connection,
    request_id: [u8; ID_BYTES],
    node_id: [u8; ID_BYTES],
    value: Vec<u8>,
) -> Result<UdpResponse, Box<dyn std::error::Error>> {
    conn.execute(
        "INSERT INTO dht_storage (key, value) VALUES (?1, ?2)",
        (node_id.to_vec(), value),
    )?;
    Ok(UdpResponse::Store { request_id })
}

pub fn find_value(
    conn: &Connection,
    key: [u8; ID_BYTES],
) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT value FROM dht_storage WHERE key = ?1")?;
    let result = stmt.query_row([key.to_vec()], |row| row.get::<_, Vec<u8>>(0));
    match result {
        Ok(value) => Ok(Some(value)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(Box::new(e)),
    }
}
