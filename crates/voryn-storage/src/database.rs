//! Database connection management and initialization.

use rusqlite::Connection;
use crate::StorageError;
use crate::schema;

/// Open (or create) an encrypted SQLCipher database.
pub fn open_database(path: &str, key: &[u8]) -> Result<Connection, StorageError> {
    let conn = Connection::open(path)?;

    // Set the encryption key for SQLCipher
    let hex_key = key.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", hex_key))?;

    // Enable WAL mode for better concurrent read performance
    conn.execute_batch("PRAGMA journal_mode = WAL;")?;

    // Enable secure delete (overwrite deleted data with zeros)
    conn.execute_batch("PRAGMA secure_delete = ON;")?;

    // Run migrations
    schema::run_migrations(&conn)?;

    Ok(conn)
}

/// Open an in-memory encrypted database (for testing).
pub fn open_memory_database() -> Result<Connection, StorageError> {
    let conn = Connection::open_in_memory()?;
    schema::run_migrations(&conn)?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_memory_database() {
        let conn = open_memory_database().unwrap();
        // Verify tables exist by querying sqlite_master
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(count > 0, "Expected tables to be created");
    }
}
