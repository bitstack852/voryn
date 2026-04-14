//! Shamir's Secret Sharing — threshold-based group key distribution.
//!
//! Splits a group encryption key into N shares, any T of which can
//! reconstruct the key. No fewer than T shares reveal any information.
//!
//! Uses GF(256) (Galois Field with 256 elements) for byte-level operations.

use rand::Rng;
use serde::{Deserialize, Serialize};

/// A single share of a split secret.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Share {
    /// Share index (1-255, never 0).
    pub index: u8,
    /// Share data (same length as the secret).
    pub data: Vec<u8>,
}

/// Split a secret into `n` shares with threshold `t`.
///
/// Any `t` shares can reconstruct the secret.
/// Fewer than `t` shares reveal nothing about the secret.
pub fn split_secret(secret: &[u8], t: u8, n: u8) -> Result<Vec<Share>, String> {
    if t < 2 {
        return Err("Threshold must be at least 2".into());
    }
    if n < t {
        return Err("Share count must be >= threshold".into());
    }
    if n == 0 {
        return Err("Share count must be > 0".into());
    }

    let mut rng = rand::thread_rng();
    let mut shares: Vec<Share> = (1..=n)
        .map(|i| Share {
            index: i,
            data: vec![0u8; secret.len()],
        })
        .collect();

    // For each byte of the secret, create a random polynomial of degree t-1
    for (byte_idx, &secret_byte) in secret.iter().enumerate() {
        // Coefficients: a[0] = secret byte, a[1..t-1] = random
        let mut coefficients = vec![0u8; t as usize];
        coefficients[0] = secret_byte;
        for coeff in coefficients.iter_mut().skip(1) {
            *coeff = rng.gen();
        }

        // Evaluate polynomial at each share index
        for share in shares.iter_mut() {
            shares_byte(share, byte_idx, &coefficients);
        }
    }

    Ok(shares)
}

/// Reconstruct a secret from `t` or more shares.
pub fn reconstruct_secret(shares: &[Share]) -> Result<Vec<u8>, String> {
    if shares.is_empty() {
        return Err("No shares provided".into());
    }

    let secret_len = shares[0].data.len();
    if shares.iter().any(|s| s.data.len() != secret_len) {
        return Err("All shares must have the same length".into());
    }

    let mut secret = vec![0u8; secret_len];

    // Lagrange interpolation in GF(256) for each byte
    for (byte_idx, byte) in secret.iter_mut().enumerate() {
        *byte = lagrange_interpolate(shares, byte_idx);
    }

    Ok(secret)
}

// ── GF(256) arithmetic ────────────────────────────────────────────

/// Evaluate polynomial at a point in GF(256) and store in share.
fn shares_byte(share: &mut Share, byte_idx: usize, coefficients: &[u8]) {
    let x = share.index;
    let mut result = 0u8;
    let mut x_power = 1u8; // x^0 = 1

    for &coeff in coefficients {
        result = gf256_add(result, gf256_mul(coeff, x_power));
        x_power = gf256_mul(x_power, x);
    }

    share.data[byte_idx] = result;
}

/// Lagrange interpolation at x=0 in GF(256).
fn lagrange_interpolate(shares: &[Share], byte_idx: usize) -> u8 {
    let mut result = 0u8;

    for i in 0..shares.len() {
        let mut basis = shares[i].data[byte_idx];

        for j in 0..shares.len() {
            if i != j {
                // basis *= x_j / (x_j - x_i)
                let xi = shares[i].index;
                let xj = shares[j].index;
                let num = xj;
                let denom = gf256_add(xj, xi); // In GF(256), subtraction = addition (XOR)
                basis = gf256_mul(basis, gf256_mul(num, gf256_inv(denom)));
            }
        }

        result = gf256_add(result, basis);
    }

    result
}

/// GF(256) addition (XOR).
fn gf256_add(a: u8, b: u8) -> u8 {
    a ^ b
}

/// GF(256) multiplication using Russian peasant algorithm.
/// Irreducible polynomial: x^8 + x^4 + x^3 + x + 1 (0x11B).
fn gf256_mul(mut a: u8, mut b: u8) -> u8 {
    let mut result: u8 = 0;
    while b > 0 {
        if b & 1 != 0 {
            result ^= a;
        }
        let carry = a & 0x80;
        a <<= 1;
        if carry != 0 {
            a ^= 0x1B; // Reduce by irreducible polynomial
        }
        b >>= 1;
    }
    result
}

/// GF(256) multiplicative inverse using extended Euclidean algorithm.
fn gf256_inv(a: u8) -> u8 {
    if a == 0 {
        return 0; // 0 has no inverse
    }
    // Use Fermat's little theorem: a^(-1) = a^(254) in GF(256)
    let mut result = a;
    for _ in 0..6 {
        result = gf256_mul(result, result);
        result = gf256_mul(result, a);
    }
    // Final squaring
    result = gf256_mul(result, result);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_reconstruct() {
        let secret = b"This is a 32-byte group key!!!!!".to_vec();
        let shares = split_secret(&secret, 3, 5).unwrap();
        assert_eq!(shares.len(), 5);

        // Reconstruct with exactly threshold shares
        let reconstructed = reconstruct_secret(&shares[..3]).unwrap();
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_any_threshold_subset_works() {
        let secret = vec![0x42u8; 32];
        let shares = split_secret(&secret, 3, 5).unwrap();

        // Try different subsets of 3 shares
        assert_eq!(reconstruct_secret(&[shares[0].clone(), shares[1].clone(), shares[2].clone()]).unwrap(), secret);
        assert_eq!(reconstruct_secret(&[shares[0].clone(), shares[2].clone(), shares[4].clone()]).unwrap(), secret);
        assert_eq!(reconstruct_secret(&[shares[1].clone(), shares[3].clone(), shares[4].clone()]).unwrap(), secret);
    }

    #[test]
    fn test_more_than_threshold_works() {
        let secret = vec![0xAB; 32];
        let shares = split_secret(&secret, 3, 5).unwrap();

        // Using all 5 shares (more than threshold 3)
        let reconstructed = reconstruct_secret(&shares).unwrap();
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_below_threshold_fails() {
        let secret = vec![0x42u8; 32];
        let shares = split_secret(&secret, 3, 5).unwrap();

        // With only 2 shares (below threshold 3), reconstruction gives wrong result
        let wrong = reconstruct_secret(&shares[..2]).unwrap();
        assert_ne!(wrong, secret);
    }

    #[test]
    fn test_gf256_arithmetic() {
        // Verify GF(256) properties
        assert_eq!(gf256_mul(0, 42), 0); // 0 * x = 0
        assert_eq!(gf256_mul(1, 42), 42); // 1 * x = x
        assert_eq!(gf256_add(42, 42), 0); // x + x = 0 in GF(256)

        // Inverse property: a * a^(-1) = 1
        for a in 1..=255u8 {
            let inv = gf256_inv(a);
            assert_eq!(gf256_mul(a, inv), 1, "Inverse failed for {}", a);
        }
    }

    #[test]
    fn test_invalid_params() {
        assert!(split_secret(b"test", 1, 5).is_err()); // threshold < 2
        assert!(split_secret(b"test", 5, 3).is_err()); // n < t
    }
}
