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
