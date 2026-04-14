//! Message model — encrypted messages stored in SQLCipher.

use rusqlite::{params, Connection};
use crate::StorageError;

#[derive(Debug, Clone)]
pub struct StoredMessage {
    pub id: i64,
    pub message_id: String,
    pub conversation_id: String,
    pub sender_pubkey: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: String,
    pub received_at: String,
    pub status: String,
    pub expires_at: Option<String>,
}

/// Insert a new message (incoming or outgoing).
#[allow(clippy::too_many_arguments)]
pub fn insert_message(
    conn: &Connection,
    message_id: &str,
    conversation_id: &str,
    sender_pubkey: &[u8],
    ciphertext: &[u8],
    nonce: &[u8],
    signature: &[u8],
    timestamp: &str,
    status: &str,
) -> Result<i64, StorageError> {
    conn.execute(
        "INSERT INTO messages (message_id, conversation_id, sender_pubkey, ciphertext, nonce, signature, timestamp, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![message_id, conversation_id, sender_pubkey, ciphertext, nonce, signature, timestamp, status],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get messages for a conversation, ordered by timestamp (newest first).
pub fn get_messages(
    conn: &Connection,
    conversation_id: &str,
    limit: u32,
    offset: u32,
) -> Result<Vec<StoredMessage>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, message_id, conversation_id, sender_pubkey, ciphertext, nonce, signature, timestamp, received_at, status, expires_at
         FROM messages WHERE conversation_id = ?1
         ORDER BY timestamp DESC LIMIT ?2 OFFSET ?3",
    )?;

    let messages = stmt
        .query_map(params![conversation_id, limit, offset], |row| {
            Ok(StoredMessage {
                id: row.get(0)?,
                message_id: row.get(1)?,
                conversation_id: row.get(2)?,
                sender_pubkey: row.get(3)?,
                ciphertext: row.get(4)?,
                nonce: row.get(5)?,
                signature: row.get(6)?,
                timestamp: row.get(7)?,
                received_at: row.get(8)?,
                status: row.get(9)?,
                expires_at: row.get(10)?,
            })
        })
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

/// Update a message's delivery status.
pub fn update_message_status(
    conn: &Connection,
    message_id: &str,
    status: &str,
) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE messages SET status = ?1 WHERE message_id = ?2",
        params![status, message_id],
    )?;
    Ok(())
}

/// Get a single message by its unique ID.
pub fn get_message_by_id(
    conn: &Connection,
    message_id: &str,
) -> Result<Option<StoredMessage>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, message_id, conversation_id, sender_pubkey, ciphertext, nonce, signature, timestamp, received_at, status, expires_at
         FROM messages WHERE message_id = ?1",
    )?;

    let result = stmt.query_row(params![message_id], |row| {
        Ok(StoredMessage {
            id: row.get(0)?,
            message_id: row.get(1)?,
            conversation_id: row.get(2)?,
            sender_pubkey: row.get(3)?,
            ciphertext: row.get(4)?,
            nonce: row.get(5)?,
            signature: row.get(6)?,
            timestamp: row.get(7)?,
            received_at: row.get(8)?,
            status: row.get(9)?,
            expires_at: row.get(10)?,
        })
    });

    match result {
        Ok(msg) => Ok(Some(msg)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(StorageError::DatabaseError(e.to_string())),
    }
}

/// Delete a single message (secure delete — SQLCipher PRAGMA secure_delete handles overwrite).
pub fn delete_message(conn: &Connection, message_id: &str) -> Result<(), StorageError> {
    conn.execute(
        "DELETE FROM messages WHERE message_id = ?1",
        params![message_id],
    )?;
    Ok(())
}

/// Delete all messages in a conversation.
pub fn delete_conversation(conn: &Connection, conversation_id: &str) -> Result<(), StorageError> {
    conn.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        params![conversation_id],
    )?;
    Ok(())
}

/// Get all pending outbound messages (for queue drain on reconnect).
pub fn get_pending_messages(conn: &Connection) -> Result<Vec<StoredMessage>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, message_id, conversation_id, sender_pubkey, ciphertext, nonce, signature, timestamp, received_at, status, expires_at
         FROM messages WHERE status = 'pending' ORDER BY timestamp ASC",
    )?;

    let messages = stmt
        .query_map([], |row| {
            Ok(StoredMessage {
                id: row.get(0)?,
                message_id: row.get(1)?,
                conversation_id: row.get(2)?,
                sender_pubkey: row.get(3)?,
                ciphertext: row.get(4)?,
                nonce: row.get(5)?,
                signature: row.get(6)?,
                timestamp: row.get(7)?,
                received_at: row.get(8)?,
                status: row.get(9)?,
                expires_at: row.get(10)?,
            })
        })
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::open_memory_database;

    #[test]
    fn test_insert_and_get_messages() {
        let conn = open_memory_database().unwrap();
        let sender = vec![1u8; 32];
        let ciphertext = vec![0xAB; 64];
        let nonce = vec![0xCD; 24];
        let sig = vec![0xEF; 64];

        insert_message(
            &conn, "msg-001", "conv-alice-bob", &sender,
            &ciphertext, &nonce, &sig, "2026-04-14T12:00:00Z", "received",
        ).unwrap();

        let msgs = get_messages(&conn, "conv-alice-bob", 50, 0).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].message_id, "msg-001");
        assert_eq!(msgs[0].ciphertext, ciphertext);
    }

    #[test]
    fn test_update_message_status() {
        let conn = open_memory_database().unwrap();
        insert_message(
            &conn, "msg-002", "conv-1", &vec![1u8; 32],
            &vec![0; 64], &vec![0; 24], &vec![0; 64],
            "2026-04-14T12:00:00Z", "pending",
        ).unwrap();

        update_message_status(&conn, "msg-002", "delivered").unwrap();
        let msg = get_message_by_id(&conn, "msg-002").unwrap().unwrap();
        assert_eq!(msg.status, "delivered");
    }

    #[test]
    fn test_get_pending_messages() {
        let conn = open_memory_database().unwrap();
        insert_message(
            &conn, "msg-p1", "conv-1", &vec![1u8; 32],
            &vec![0; 64], &vec![0; 24], &vec![0; 64],
            "2026-04-14T12:00:00Z", "pending",
        ).unwrap();
        insert_message(
            &conn, "msg-d1", "conv-1", &vec![1u8; 32],
            &vec![0; 64], &vec![0; 24], &vec![0; 64],
            "2026-04-14T12:01:00Z", "delivered",
        ).unwrap();

        let pending = get_pending_messages(&conn).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].message_id, "msg-p1");
    }

    #[test]
    fn test_delete_message() {
        let conn = open_memory_database().unwrap();
        insert_message(
            &conn, "msg-del", "conv-1", &vec![1u8; 32],
            &vec![0; 64], &vec![0; 24], &vec![0; 64],
            "2026-04-14T12:00:00Z", "received",
        ).unwrap();

        delete_message(&conn, "msg-del").unwrap();
        let msg = get_message_by_id(&conn, "msg-del").unwrap();
        assert!(msg.is_none());
    }
}
