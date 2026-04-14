//! Contact model — known peers stored in SQLCipher.

use rusqlite::{params, Connection};
use crate::StorageError;

#[derive(Debug, Clone)]
pub struct StoredContact {
    pub id: i64,
    pub public_key: Vec<u8>,
    pub public_key_hex: String,
    pub display_name: Option<String>,
    pub added_at: String,
    pub last_seen: Option<String>,
    pub is_blocked: bool,
    pub is_verified: bool,
}

/// Insert a new contact.
pub fn insert_contact(
    conn: &Connection,
    public_key: &[u8],
    public_key_hex: &str,
    display_name: Option<&str>,
) -> Result<i64, StorageError> {
    conn.execute(
        "INSERT INTO contacts (public_key, public_key_hex, display_name) VALUES (?1, ?2, ?3)",
        params![public_key, public_key_hex, display_name],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get all non-blocked contacts.
pub fn get_contacts(conn: &Connection) -> Result<Vec<StoredContact>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, public_key, public_key_hex, display_name, added_at, last_seen, is_blocked, is_verified
         FROM contacts WHERE is_blocked = 0 ORDER BY display_name, public_key_hex",
    )?;

    let contacts = stmt
        .query_map([], |row| {
            Ok(StoredContact {
                id: row.get(0)?,
                public_key: row.get(1)?,
                public_key_hex: row.get(2)?,
                display_name: row.get(3)?,
                added_at: row.get(4)?,
                last_seen: row.get(5)?,
                is_blocked: row.get(6)?,
                is_verified: row.get(7)?,
            })
        })
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(contacts)
}

/// Find a contact by public key hex.
pub fn get_contact_by_pubkey(
    conn: &Connection,
    public_key_hex: &str,
) -> Result<Option<StoredContact>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, public_key, public_key_hex, display_name, added_at, last_seen, is_blocked, is_verified
         FROM contacts WHERE public_key_hex = ?1",
    )?;

    let result = stmt
        .query_row(params![public_key_hex], |row| {
            Ok(StoredContact {
                id: row.get(0)?,
                public_key: row.get(1)?,
                public_key_hex: row.get(2)?,
                display_name: row.get(3)?,
                added_at: row.get(4)?,
                last_seen: row.get(5)?,
                is_blocked: row.get(6)?,
                is_verified: row.get(7)?,
            })
        });

    match result {
        Ok(contact) => Ok(Some(contact)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(StorageError::DatabaseError(e.to_string())),
    }
}

/// Update a contact's display name.
pub fn update_contact_name(
    conn: &Connection,
    public_key_hex: &str,
    display_name: &str,
) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE contacts SET display_name = ?1 WHERE public_key_hex = ?2",
        params![display_name, public_key_hex],
    )?;
    Ok(())
}

/// Update a contact's last seen timestamp.
pub fn update_last_seen(
    conn: &Connection,
    public_key_hex: &str,
    last_seen: &str,
) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE contacts SET last_seen = ?1 WHERE public_key_hex = ?2",
        params![last_seen, public_key_hex],
    )?;
    Ok(())
}

/// Block a contact (they can no longer send us messages).
pub fn block_contact(conn: &Connection, public_key_hex: &str) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE contacts SET is_blocked = 1 WHERE public_key_hex = ?1",
        params![public_key_hex],
    )?;
    Ok(())
}

/// Delete a contact entirely.
pub fn delete_contact(conn: &Connection, public_key_hex: &str) -> Result<(), StorageError> {
    conn.execute(
        "DELETE FROM contacts WHERE public_key_hex = ?1",
        params![public_key_hex],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::open_memory_database;

    #[test]
    fn test_insert_and_get_contact() {
        let conn = open_memory_database().unwrap();
        let pk = vec![2u8; 32];
        let hex = "02".repeat(32);
        insert_contact(&conn, &pk, &hex, Some("Alice")).unwrap();

        let contacts = get_contacts(&conn).unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].display_name, Some("Alice".to_string()));
    }

    #[test]
    fn test_get_contact_by_pubkey() {
        let conn = open_memory_database().unwrap();
        let hex = "03".repeat(32);
        insert_contact(&conn, &vec![3u8; 32], &hex, Some("Bob")).unwrap();

        let contact = get_contact_by_pubkey(&conn, &hex).unwrap().unwrap();
        assert_eq!(contact.display_name, Some("Bob".to_string()));

        let missing = get_contact_by_pubkey(&conn, "nonexistent").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_block_contact() {
        let conn = open_memory_database().unwrap();
        let hex = "04".repeat(32);
        insert_contact(&conn, &vec![4u8; 32], &hex, Some("Eve")).unwrap();

        block_contact(&conn, &hex).unwrap();
        let contacts = get_contacts(&conn).unwrap();
        assert!(contacts.is_empty()); // Blocked contacts excluded
    }
}
