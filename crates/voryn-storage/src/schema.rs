//! Database schema definitions and migrations.

use rusqlite::Connection;
use crate::StorageError;

/// Current schema version.
pub const SCHEMA_VERSION: i32 = 1;

/// Run all pending migrations to bring the database up to the current schema.
pub fn run_migrations(conn: &Connection) -> Result<(), StorageError> {
    let current_version = get_schema_version(conn)?;

    if current_version < 1 {
        migrate_v1(conn)?;
    }

    Ok(())
}

fn get_schema_version(conn: &Connection) -> Result<i32, StorageError> {
    let version: i32 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| StorageError::MigrationError(e.to_string()))?;
    Ok(version)
}

fn set_schema_version(conn: &Connection, version: i32) -> Result<(), StorageError> {
    conn.execute_batch(&format!("PRAGMA user_version = {};", version))
        .map_err(|e| StorageError::MigrationError(e.to_string()))?;
    Ok(())
}

/// Schema v1: Core tables for identity, contacts, messages, key material.
fn migrate_v1(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        "
        -- Local device identity
        CREATE TABLE IF NOT EXISTS identities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_key BLOB NOT NULL UNIQUE,
            public_key_hex TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            is_active INTEGER NOT NULL DEFAULT 1
        );

        -- Known contacts
        CREATE TABLE IF NOT EXISTS contacts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            public_key BLOB NOT NULL UNIQUE,
            public_key_hex TEXT NOT NULL,
            display_name TEXT,
            added_at TEXT NOT NULL DEFAULT (datetime('now')),
            last_seen TEXT,
            is_blocked INTEGER NOT NULL DEFAULT 0,
            is_verified INTEGER NOT NULL DEFAULT 0
        );

        -- Messages (encrypted at rest via SQLCipher)
        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id TEXT NOT NULL UNIQUE,
            conversation_id TEXT NOT NULL,
            sender_pubkey BLOB NOT NULL,
            ciphertext BLOB NOT NULL,
            nonce BLOB NOT NULL,
            signature BLOB NOT NULL,
            timestamp TEXT NOT NULL,
            received_at TEXT NOT NULL DEFAULT (datetime('now')),
            status TEXT NOT NULL DEFAULT 'received',
            expires_at TEXT
        );

        -- Key material for active ratchet sessions
        CREATE TABLE IF NOT EXISTS key_material (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            peer_pubkey BLOB NOT NULL,
            session_data BLOB NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(peer_pubkey)
        );

        -- Indexes for common queries
        CREATE INDEX IF NOT EXISTS idx_messages_conversation
            ON messages(conversation_id, timestamp);
        CREATE INDEX IF NOT EXISTS idx_messages_status
            ON messages(status);
        CREATE INDEX IF NOT EXISTS idx_contacts_pubkey
            ON contacts(public_key_hex);
        ",
    )
    .map_err(|e| StorageError::MigrationError(e.to_string()))?;

    set_schema_version(conn, 1)?;
    tracing::info!("Database migrated to schema v1");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap(); // Running again should be a no-op
        assert_eq!(get_schema_version(&conn).unwrap(), SCHEMA_VERSION);
    }
}
