//! Content hashes so decisions bind topology, not mutable display names.

use sha2::{Digest, Sha256};

/// Stable hex SHA-256 of an arbitrary byte payload.
pub fn content_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_payload_is_deterministic() {
        assert_eq!(
            content_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
