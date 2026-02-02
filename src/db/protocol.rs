use rusqlite::Connection;

pub fn store (
    conn: Connection,
    key: String,
    file_name: String,
    file_path: String,
    owner_ip: String,
    owner_port: String,
    file_size: u32,
) -> Result<(), Box<dyn std::error::Error>>{

    conn.execute(
        "INSERT INTO dht_storage (key, file_name, file_path, owner_ip, owner_port, file_size)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (key, file_name, file_path, owner_ip, owner_port,file_size),
    )?;

    Ok(())
}

pub fn find_node(node_id: String) {

}
