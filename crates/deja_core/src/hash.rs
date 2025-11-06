//! Hash-based clone detection utilities

use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

/// Fast hash function for token sequences
pub fn hash_tokens(tokens: &[String]) -> u64 {
    let mut hasher = FxHasher::default();
    for token in tokens {
        token.hash(&mut hasher);
    }
    hasher.finish()
}

/// Rolling hash for sliding window detection
pub struct RollingHash {
    base: u64,
    modulus: u64,
    window_size: usize,
    current_hash: u64,
    window: Vec<u64>,
}

impl RollingHash {
    pub fn new(window_size: usize) -> Self {
        Self {
            base: 31,
            modulus: 1_000_000_007,
            window_size,
            current_hash: 0,
            window: Vec::with_capacity(window_size),
        }
    }

    /// Add a token to the rolling hash
    pub fn push(&mut self, token: &str) -> Option<u64> {
        let token_hash = self.hash_string(token);

        if self.window.len() < self.window_size {
            // Building initial window
            self.current_hash = (self.current_hash * self.base + token_hash) % self.modulus;
            self.window.push(token_hash);

            if self.window.len() == self.window_size {
                Some(self.current_hash)
            } else {
                None
            }
        } else {
            // Sliding window
            let old_token = self.window.remove(0);
            self.window.push(token_hash);

            // Remove old token contribution
            let base_power = self.mod_pow(self.base, self.window_size as u64 - 1);
            self.current_hash = (self.current_hash + self.modulus - (old_token * base_power) % self.modulus) % self.modulus;

            // Add new token
            self.current_hash = (self.current_hash * self.base + token_hash) % self.modulus;

            Some(self.current_hash)
        }
    }

    fn hash_string(&self, s: &str) -> u64 {
        let mut hasher = FxHasher::default();
        s.hash(&mut hasher);
        hasher.finish() % self.modulus
    }

    fn mod_pow(&self, base: u64, mut exp: u64) -> u64 {
        let mut result = 1u64;
        let mut base = base % self.modulus;

        while exp > 0 {
            if exp % 2 == 1 {
                result = (result * base) % self.modulus;
            }
            exp >>= 1;
            base = (base * base) % self.modulus;
        }

        result
    }

    pub fn reset(&mut self) {
        self.current_hash = 0;
        self.window.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_tokens() {
        let tokens1 = vec!["def".to_string(), "foo".to_string()];
        let tokens2 = vec!["def".to_string(), "foo".to_string()];
        let tokens3 = vec!["def".to_string(), "bar".to_string()];

        assert_eq!(hash_tokens(&tokens1), hash_tokens(&tokens2));
        assert_ne!(hash_tokens(&tokens1), hash_tokens(&tokens3));
    }

    #[test]
    fn test_rolling_hash() {
        let mut roller = RollingHash::new(3);

        assert!(roller.push("def").is_none());
        assert!(roller.push("foo").is_none());
        let hash1 = roller.push("bar");
        assert!(hash1.is_some());

        let hash2 = roller.push("baz");
        assert!(hash2.is_some());
        assert_ne!(hash1, hash2);
    }
}
