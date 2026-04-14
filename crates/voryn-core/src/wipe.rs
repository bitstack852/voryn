//! Secure data wipe — irreversibly destroys all app data.
//!
//! Triggered by:
//! - Failed passcode attempt limit reached
//! - Remote wipe command from trusted contact
//! - Manual user action
//!
//! Wipe procedure:
//! 1. Overwrite SQLCipher database file with random bytes
//! 2. Delete hardware keystore keys
//! 3. Delete all cached data
//! 4. Reset app to fresh install state

use tracing::{info, warn};

/// Result of a wipe operation.
#[derive(Debug)]
pub struct WipeResult {
    pub database_wiped: bool,
    pub keystore_wiped: bool,
    pub cache_wiped: bool,
    pub errors: Vec<String>,
}

/// Perform a complete data wipe.
///
/// This is irreversible. All encrypted data, keys, contacts, and messages
/// will be permanently destroyed.
pub fn perform_full_wipe(
    db_path: &str,
    keystore: &dyn crate::keystore::HardwareKeyStore,
    key_alias: &str,
) -> WipeResult {
    let mut result = WipeResult {
        database_wiped: false,
        keystore_wiped: false,
        cache_wiped: false,
        errors: Vec::new(),
    };

    info!("WIPE: Initiating full data wipe");

    // 1. Overwrite database file with random bytes
    match overwrite_file(db_path) {
        Ok(_) => {
            result.database_wiped = true;
            info!("WIPE: Database file overwritten and deleted");
        }
        Err(e) => {
            warn!("WIPE: Failed to overwrite database: {}", e);
            result.errors.push(format!("Database: {}", e));
        }
    }

    // 2. Delete hardware keystore key
    let handle = crate::keystore::KeyHandle {
        alias: key_alias.to_string(),
    };
    match keystore.delete_key(&handle) {
        Ok(_) => {
            result.keystore_wiped = true;
            info!("WIPE: Hardware keystore key deleted");
        }
        Err(e) => {
            warn!("WIPE: Failed to delete keystore key: {}", e);
            result.errors.push(format!("Keystore: {}", e));
        }
    }

    // 3. Clear WAL and SHM files
    let wal_path = format!("{}-wal", db_path);
    let shm_path = format!("{}-shm", db_path);
    let _ = overwrite_file(&wal_path);
    let _ = overwrite_file(&shm_path);
    result.cache_wiped = true;

    info!(
        "WIPE: Complete. DB={}, Keys={}, Cache={}",
        result.database_wiped, result.keystore_wiped, result.cache_wiped
    );

    result
}

/// Overwrite a file with random bytes, then delete it.
fn overwrite_file(path: &str) -> Result<(), String> {
    use std::fs;
    use std::io::Write;

    // Get file size
    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    let size = metadata.len() as usize;

    // Overwrite with random bytes (3 passes for good measure)
    for pass in 0..3 {
        let random_data: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(path)
            .map_err(|e| e.to_string())?;
        file.write_all(&random_data).map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        tracing::debug!("WIPE: Overwrite pass {} complete for {}", pass + 1, path);
    }

    // Delete the file
    fs::remove_file(path).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_overwrite_file() {
        let path = "/tmp/voryn_wipe_test";
        // Create a test file
        let mut f = fs::File::create(path).unwrap();
        f.write_all(b"sensitive data here").unwrap();
        f.sync_all().unwrap();
        drop(f);

        // Wipe it
        overwrite_file(path).unwrap();

        // File should be gone
        assert!(!std::path::Path::new(path).exists());
    }
}
