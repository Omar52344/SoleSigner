use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hex;

use sha2::{Digest, Sha256};

/// Generates a unique nullifier: SHA256(Document + Election_Salt)
pub fn generate_nullifier(document: &str, election_salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(document.as_bytes());
    hasher.update(election_salt.as_bytes());
    hex::encode(hasher.finalize())
}

/// Computes a standard SHA256 hash of the input
pub fn hash_data(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

// Basic Merkle Tree Implementation
#[derive(Debug, Clone)]
pub struct MerkleTree {
    pub leaves: Vec<String>,
    pub root: String,
    pub levels: Vec<Vec<String>>,
}

impl MerkleTree {
    pub fn new(leaves: Vec<String>) -> Self {
        if leaves.is_empty() {
            return MerkleTree {
                leaves: vec![],
                root: String::new(),
                levels: vec![],
            };
        }

        let mut current_level = leaves.clone();
        let mut levels = vec![current_level.clone()];

        while current_level.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in current_level.chunks(2) {
                let left = &chunk[0];
                let right = if chunk.len() > 1 {
                    &chunk[1]
                } else {
                    &chunk[0]
                }; // Duplicate if odd

                let combined = format!("{}{}", left, right);
                next_level.push(hash_data(&combined));
            }

            levels.push(next_level.clone());
            current_level = next_level;
        }

        MerkleTree {
            leaves,
            root: current_level[0].clone(),
            levels,
        }
    }

    #[allow(clippy::manual_is_multiple_of)]
    pub fn get_proof(&self, index: usize) -> Vec<String> {
        let mut proof = Vec::new();
        let mut current_index = index;

        // Skip the root level (last level)
        for level in self.levels.iter().take(self.levels.len() - 1) {
            let is_left = current_index % 2 == 0;
            let pair_index = if is_left {
                current_index + 1
            } else {
                current_index - 1
            };

            if pair_index < level.len() {
                proof.push(level[pair_index].clone());
            } else {
                // Even number at end, pair is itself
                proof.push(level[current_index].clone());
            }

            current_index /= 2;
        }
        proof
    }
}

/// Derives an Ed25519 signing key from a master secret and election salt.
pub fn derive_election_signing_key(master_secret: &str, election_salt: &str) -> SigningKey {
    let mut hasher = Sha256::new();
    hasher.update(master_secret.as_bytes());
    hasher.update(election_salt.as_bytes());
    let seed = hasher.finalize();

    // Ensure the seed is 32 bytes (SHA256 output is 32 bytes)
    let seed_array: [u8; 32] = seed.into();
    SigningKey::from_bytes(&seed_array)
}

/// Signs a message with a signing key and returns the signature as hex string.
pub fn sign_message(signing_key: &SigningKey, message: &str) -> String {
    let signature = signing_key.sign(message.as_bytes());
    hex::encode(signature.to_bytes())
}

/// Verifies a signature against a message using a verifying key.
pub fn verify_signature(verifying_key: &VerifyingKey, message: &str, signature_hex: &str) -> bool {
    let signature_bytes = match hex::decode(signature_hex) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    if signature_bytes.len() != 64 {
        return false;
    }
    let signature_array: [u8; 64] = signature_bytes.try_into().unwrap();
    let signature = Signature::from_bytes(&signature_array);
    verifying_key.verify(message.as_bytes(), &signature).is_ok()
}
