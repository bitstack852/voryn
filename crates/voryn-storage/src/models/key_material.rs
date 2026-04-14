//! Key material model — stores ratchet session state and pre-keys.

use rusqlite::{params, Connection};
use crate::StorageError;

/// Store or update session key material for a peer.
pub fn upsert_key_material(
    conn: &Connection,
    peer_pubkey: &[u8],
    session_data: &[u8],
) -> Result<(), StorageError> {
    conn.execute(
        "INSERT INTO key_material (peer_pubkey, session_data, updated_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(peer_pubkey)
         DO UPDATE SET session_data = ?2, updated_at = datetime('now')",
        params![peer_pubkey, session_data],
    )?;
    Ok(())
}

/// Get session key material for a peer.
pub fn get_key_material(
    conn: &Connection,
    peer_pubkey: &[u8],
) -> Result<Option<Vec<u8>>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT session_data FROM key_material WHERE peer_pubkey = ?1",
    )?;

    let result = stmt.query_row(params![peer_pubkey], |row| row.get(0));

    match result {
        Ok(data) => Ok(Some(data)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(StorageError::DatabaseError(e.to_string())),
    }
}

/// Delete key material for a peer (on contact removal or identity revocation).
pub fn delete_key_material(
    conn: &Connection,
    peer_pubkey: &[u8],
) -> Result<(), StorageError> {
    conn.execute(
        "DELETE FROM key_material WHERE peer_pubkey = ?1",
        params![peer_pubkey],
    )?;
    Ok(())
}

/// Delete all key material (full wipe).
pub fn delete_all_key_material(conn: &Connection) -> Result<(), StorageError> {
    conn.execute("DELETE FROM key_material", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::open_memory_database;

    #[test]
    fn test_upsert_and_get() {
        let conn = open_memory_database().unwrap();
        let peer = vec![5u8; 32];
        let data = vec![0xAA; 128];

        upsert_key_material(&conn, &peer, &data).unwrap();
        let retrieved = get_key_material(&conn, &peer).unwrap().unwrap();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_upsert_overwrites() {
        let conn = open_memory_database().unwrap();
        let peer = vec![6u8; 32];

        upsert_key_material(&conn, &peer, &vec![1; 64]).unwrap();
        upsert_key_material(&conn, &peer, &vec![2; 64]).unwrap();

        let retrieved = get_key_material(&conn, &peer).unwrap().unwrap();
        assert_eq!(retrieved, vec![2; 64]);
    }

    #[test]
    fn test_missing_key_material() {
        let conn = open_memory_database().unwrap();
        let result = get_key_material(&conn, &vec![99u8; 32]).unwrap();
        assert!(result.is_none());
    }
}
