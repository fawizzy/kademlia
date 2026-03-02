use rusqlite::Connection;

use crate::utils::{constant::ID_BYTES};



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
