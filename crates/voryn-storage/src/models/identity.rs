//! Identity model — local device identity stored in SQLCipher.

use rusqlite::{params, Connection};
use crate::StorageError;

#[derive(Debug, Clone)]
pub struct StoredIdentity {
    pub id: i64,
    pub public_key: Vec<u8>,
    pub public_key_hex: String,
    pub created_at: String,
    pub is_active: bool,
}

/// Store a new identity in the database.
pub fn insert_identity(
    conn: &Connection,
    public_key: &[u8],
    public_key_hex: &str,
) -> Result<i64, StorageError> {
    conn.execute(
        "INSERT INTO identities (public_key, public_key_hex) VALUES (?1, ?2)",
        params![public_key, public_key_hex],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get the active identity.
pub fn get_active_identity(conn: &Connection) -> Result<Option<StoredIdentity>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, public_key, public_key_hex, created_at, is_active
         FROM identities WHERE is_active = 1 LIMIT 1",
    )?;

    let result = stmt
        .query_row([], |row| {
            Ok(StoredIdentity {
                id: row.get(0)?,
                public_key: row.get(1)?,
                public_key_hex: row.get(2)?,
                created_at: row.get(3)?,
                is_active: row.get(4)?,
            })
        })
        .optional()
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

    Ok(result)
}

/// Trait extension for optional query results.
trait OptionalExt<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::open_memory_database;

    #[test]
    fn test_insert_and_get_identity() {
        let conn = open_memory_database().unwrap();
        let pk = vec![1u8; 32];
        let hex = "01".repeat(32);
        let id = insert_identity(&conn, &pk, &hex).unwrap();
        assert!(id > 0);

        let identity = get_active_identity(&conn).unwrap().unwrap();
        assert_eq!(identity.public_key, pk);
        assert_eq!(identity.public_key_hex, hex);
        assert!(identity.is_active);
    }
}
