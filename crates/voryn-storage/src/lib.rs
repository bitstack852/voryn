//! Voryn Storage — Encrypted local database built on SQLCipher.
//!
//! All user data (identities, contacts, messages, key material) is stored
//! in a SQLCipher-encrypted database. The encryption key is derived from
//! the hardware-bound device key.

pub mod database;
pub mod schema;
pub mod models;
pub mod migrations;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        StorageError::DatabaseError(err.to_string())
    }
}
