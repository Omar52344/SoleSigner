use sha2::{Digest, Sha256};
use hex;

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
                let right = if chunk.len() > 1 { &chunk[1] } else { &chunk[0] }; // Duplicate if odd
                
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

    pub fn get_proof(&self, index: usize) -> Vec<String> {
        let mut proof = Vec::new();
        let mut current_index = index;

        // Skip the root level (last level)
        for level in self.levels.iter().take(self.levels.len() - 1) {
             let is_left = current_index % 2 == 0;
             let pair_index = if is_left { current_index + 1 } else { current_index - 1 };
             
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
