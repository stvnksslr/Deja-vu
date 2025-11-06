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
    fn test_hash_tokens_empty() {
        let tokens: Vec<String> = vec![];
        let hash = hash_tokens(&tokens);
        // Empty should produce consistent hash
        assert_eq!(hash, hash_tokens(&tokens));
    }

    #[test]
    fn test_hash_tokens_order_matters() {
        let tokens1 = vec!["foo".to_string(), "bar".to_string()];
        let tokens2 = vec!["bar".to_string(), "foo".to_string()];

        // Different order should produce different hash
        assert_ne!(hash_tokens(&tokens1), hash_tokens(&tokens2));
    }

    #[test]
    fn test_hash_tokens_deterministic() {
        let tokens = vec!["def".to_string(), "foo".to_string(), "bar".to_string()];

        let hash1 = hash_tokens(&tokens);
        let hash2 = hash_tokens(&tokens);
        let hash3 = hash_tokens(&tokens);

        assert_eq!(hash1, hash2);
        assert_eq!(hash2, hash3);
    }

    #[test]
    fn test_rolling_hash_window_size_1() {
        let mut roller = RollingHash::new(1);

        let hash1 = roller.push("token1");
        assert!(hash1.is_some());

        let hash2 = roller.push("token2");
        assert!(hash2.is_some());
        assert_ne!(hash1, hash2);
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

    #[test]
    fn test_rolling_hash_same_sequence() {
        let mut roller1 = RollingHash::new(3);
        let mut roller2 = RollingHash::new(3);

        let sequence = vec!["def", "foo", "bar", "baz"];

        let mut hashes1 = Vec::new();
        let mut hashes2 = Vec::new();

        for token in &sequence {
            if let Some(hash) = roller1.push(token) {
                hashes1.push(hash);
            }
            if let Some(hash) = roller2.push(token) {
                hashes2.push(hash);
            }
        }

        assert_eq!(hashes1, hashes2, "Same sequence should produce same hashes");
    }

    #[test]
    fn test_rolling_hash_reset() {
        let mut roller = RollingHash::new(3);

        roller.push("def");
        roller.push("foo");
        let hash1 = roller.push("bar");

        roller.reset();

        roller.push("def");
        roller.push("foo");
        let hash2 = roller.push("bar");

        assert_eq!(hash1, hash2, "Reset should allow recomputing same hash");
    }

    #[test]
    fn test_rolling_hash_sliding_window() {
        let mut roller = RollingHash::new(3);

        // Build initial window: [A, B, C]
        roller.push("A");
        roller.push("B");
        let hash_abc = roller.push("C");

        // Slide to: [B, C, D]
        let hash_bcd = roller.push("D");

        // Slide to: [C, D, E]
        let hash_cde = roller.push("E");

        assert!(hash_abc.is_some());
        assert!(hash_bcd.is_some());
        assert!(hash_cde.is_some());

        // All should be different
        assert_ne!(hash_abc, hash_bcd);
        assert_ne!(hash_bcd, hash_cde);
        assert_ne!(hash_abc, hash_cde);
    }

    #[test]
    fn test_rolling_hash_repeated_pattern() {
        let mut roller = RollingHash::new(2);

        let hash1 = roller.push("X");
        let hash2 = roller.push("Y");
        let hash3 = roller.push("X");
        let hash4 = roller.push("Y");

        // [X, Y] and [X, Y] should produce same hash
        assert_eq!(hash2, hash4);
    }

    #[test]
    fn test_rolling_hash_large_window() {
        let mut roller = RollingHash::new(10);

        for i in 0..9 {
            assert!(roller.push(&format!("token{}", i)).is_none());
        }

        let hash = roller.push("token9");
        assert!(hash.is_some(), "Should produce hash after filling window");
    }

    #[test]
    fn test_rolling_hash_special_characters() {
        let mut roller = RollingHash::new(2);

        roller.push("@#$");
        let hash1 = roller.push("!%^");

        roller.reset();

        roller.push("@#$");
        let hash2 = roller.push("!%^");

        assert_eq!(hash1, hash2, "Should handle special characters consistently");
    }

    #[test]
    fn test_rolling_hash_unicode() {
        let mut roller = RollingHash::new(2);

        roller.push("你好");
        let hash = roller.push("世界");

        assert!(hash.is_some(), "Should handle Unicode characters");
    }
}
