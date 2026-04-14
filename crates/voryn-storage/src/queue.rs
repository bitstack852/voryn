//! Persistent outbound message queue — ensures delivery even when recipients are offline.
//!
//! Messages are queued on send and dequeued only after delivery confirmation (ACK).
//! Queue persists across app restarts via SQLCipher.

use rusqlite::{params, Connection};
use crate::StorageError;

/// A queued outbound message awaiting delivery.
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub id: i64,
    pub message_id: String,
    pub recipient_pubkey: Vec<u8>,
    pub payload: Vec<u8>, // Serialized EncryptedMessage
    pub created_at: String,
    pub retry_count: i32,
    pub next_retry_at: String,
    pub status: String, // pending, sending, sent, failed
}

/// Create the queue table (called during migration).
pub fn create_queue_table(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS outbound_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            message_id TEXT NOT NULL UNIQUE,
            recipient_pubkey BLOB NOT NULL,
            payload BLOB NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            retry_count INTEGER NOT NULL DEFAULT 0,
            next_retry_at TEXT NOT NULL DEFAULT (datetime('now')),
            status TEXT NOT NULL DEFAULT 'pending'
        );
        CREATE INDEX IF NOT EXISTS idx_queue_status ON outbound_queue(status, next_retry_at);",
    )?;
    Ok(())
}

/// Enqueue a message for delivery.
pub fn enqueue(
    conn: &Connection,
    message_id: &str,
    recipient_pubkey: &[u8],
    payload: &[u8],
) -> Result<i64, StorageError> {
    conn.execute(
        "INSERT INTO outbound_queue (message_id, recipient_pubkey, payload) VALUES (?1, ?2, ?3)",
        params![message_id, recipient_pubkey, payload],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Get all pending messages ready for retry.
pub fn get_pending(conn: &Connection) -> Result<Vec<QueuedMessage>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, message_id, recipient_pubkey, payload, created_at, retry_count, next_retry_at, status
         FROM outbound_queue
         WHERE status = 'pending' AND next_retry_at <= datetime('now')
         ORDER BY created_at ASC",
    )?;

    let msgs = stmt
        .query_map([], |row| {
            Ok(QueuedMessage {
                id: row.get(0)?,
                message_id: row.get(1)?,
                recipient_pubkey: row.get(2)?,
                payload: row.get(3)?,
                created_at: row.get(4)?,
                retry_count: row.get(5)?,
                next_retry_at: row.get(6)?,
                status: row.get(7)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(msgs)
}

/// Mark a message as sent (awaiting ACK).
pub fn mark_sent(conn: &Connection, message_id: &str) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE outbound_queue SET status = 'sent' WHERE message_id = ?1",
        params![message_id],
    )?;
    Ok(())
}

/// Mark a message as delivered (remove from queue).
pub fn mark_delivered(conn: &Connection, message_id: &str) -> Result<(), StorageError> {
    conn.execute(
        "DELETE FROM outbound_queue WHERE message_id = ?1",
        params![message_id],
    )?;
    Ok(())
}

/// Increment retry count and set next retry time with exponential backoff.
/// Backoff: 1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s, capped at 300s (5 min).
pub fn retry_failed(conn: &Connection, message_id: &str) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE outbound_queue
         SET retry_count = retry_count + 1,
             status = 'pending',
             next_retry_at = datetime('now', '+' || MIN(POWER(2, retry_count), 300) || ' seconds')
         WHERE message_id = ?1",
        params![message_id],
    )?;
    Ok(())
}

/// Mark a message as permanently failed.
pub fn mark_failed(conn: &Connection, message_id: &str) -> Result<(), StorageError> {
    conn.execute(
        "UPDATE outbound_queue SET status = 'failed' WHERE message_id = ?1",
        params![message_id],
    )?;
    Ok(())
}

/// Get count of pending messages for a recipient.
pub fn pending_count_for(conn: &Connection, recipient_pubkey: &[u8]) -> Result<i64, StorageError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM outbound_queue WHERE recipient_pubkey = ?1 AND status = 'pending'",
        params![recipient_pubkey],
        |row| row.get(0),
    )?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::open_memory_database;

    fn setup_db() -> Connection {
        let conn = open_memory_database().unwrap();
        create_queue_table(&conn).unwrap();
        conn
    }

    #[test]
    fn test_enqueue_and_get_pending() {
        let conn = setup_db();
        enqueue(&conn, "msg-q1", &[1u8; 32], &[0xAB; 100]).unwrap();
        let pending = get_pending(&conn).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].message_id, "msg-q1");
    }

    #[test]
    fn test_mark_delivered_removes() {
        let conn = setup_db();
        enqueue(&conn, "msg-q2", &[2u8; 32], &[0; 50]).unwrap();
        mark_delivered(&conn, "msg-q2").unwrap();
        let pending = get_pending(&conn).unwrap();
        assert!(pending.is_empty());
    }

    #[test]
    fn test_mark_sent() {
        let conn = setup_db();
        enqueue(&conn, "msg-q3", &[3u8; 32], &[0; 50]).unwrap();
        mark_sent(&conn, "msg-q3").unwrap();
        // Sent messages shouldn't appear in pending
        let pending = get_pending(&conn).unwrap();
        assert!(pending.is_empty());
    }
}
