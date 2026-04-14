//! Message padding to fixed-size buckets to prevent length analysis.
//!
//! All messages are padded to the nearest bucket size before transmission.
//! An observer cannot determine the actual message length from packet size.

/// Fixed padding bucket sizes (bytes).
const BUCKET_SIZES: &[usize] = &[256, 1024, 4096, 16384, 65536];

/// Pad a message to the nearest bucket size.
/// Returns padded data with the original length prepended as a 4-byte big-endian prefix.
pub fn pad_message(data: &[u8]) -> Vec<u8> {
    let original_len = data.len();
    let padded_size = select_bucket(original_len + 4); // +4 for length prefix

    let mut padded = Vec::with_capacity(padded_size);
    // Prepend original length as 4-byte big-endian
    padded.extend_from_slice(&(original_len as u32).to_be_bytes());
    padded.extend_from_slice(data);
    // Fill remainder with random bytes
    let remaining = padded_size - padded.len();
    let random_padding: Vec<u8> = (0..remaining).map(|_| rand::random::<u8>()).collect();
    padded.extend_from_slice(&random_padding);

    padded
}

/// Remove padding from a received message. Returns the original data.
pub fn unpad_message(padded: &[u8]) -> Result<Vec<u8>, String> {
    if padded.len() < 4 {
        return Err("Padded message too short".into());
    }

    let original_len = u32::from_be_bytes([padded[0], padded[1], padded[2], padded[3]]) as usize;

    if original_len + 4 > padded.len() {
        return Err("Invalid padding: declared length exceeds data".into());
    }

    Ok(padded[4..4 + original_len].to_vec())
}

/// Select the smallest bucket that fits the data.
fn select_bucket(data_len: usize) -> usize {
    for &size in BUCKET_SIZES {
        if data_len <= size {
            return size;
        }
    }
    // For very large messages, round up to nearest 64KB multiple
    data_len.div_ceil(65536) * 65536
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pad_unpad_roundtrip() {
        let original = b"Hello, this is a secret message!";
        let padded = pad_message(original);
        assert_eq!(padded.len(), 256); // Smallest bucket
        let recovered = unpad_message(&padded).unwrap();
        assert_eq!(recovered, original);
    }

    #[test]
    fn test_bucket_selection() {
        // Small message → 256 bytes
        assert_eq!(pad_message(b"hi").len(), 256);
        // 300 bytes → 1024 bucket
        let data = vec![0u8; 300];
        assert_eq!(pad_message(&data).len(), 1024);
        // 2000 bytes → 4096 bucket
        let data = vec![0u8; 2000];
        assert_eq!(pad_message(&data).len(), 4096);
    }

    #[test]
    fn test_all_padded_same_size_are_indistinguishable() {
        let msg1 = pad_message(b"short");
        let msg2 = pad_message(b"a slightly longer message");
        let msg3 = pad_message(b"yet another message of different length!!!");
        // All fit in 256-byte bucket
        assert_eq!(msg1.len(), msg2.len());
        assert_eq!(msg2.len(), msg3.len());
    }

    #[test]
    fn test_invalid_padding_rejected() {
        assert!(unpad_message(b"ab").is_err()); // Too short
        // Declared length exceeds actual data
        let mut bad = vec![0xFF, 0xFF, 0xFF, 0xFF]; // claims 4GB
        bad.extend_from_slice(b"small");
        assert!(unpad_message(&bad).is_err());
    }
}
