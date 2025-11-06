//! Similarity metrics for comparing code fragments

use crate::token::TokenSequence;

/// Trait for calculating similarity between code fragments
pub trait SimilarityMetric: Send + Sync {
    fn calculate(&self, a: &TokenSequence, b: &TokenSequence) -> f64;
    fn name(&self) -> &str;
}

/// Jaccard similarity coefficient
pub struct JaccardSimilarity;

impl SimilarityMetric for JaccardSimilarity {
    fn calculate(&self, a: &TokenSequence, b: &TokenSequence) -> f64 {
        let set_a: std::collections::HashSet<_> =
            a.tokens.iter().map(|t| t.normalize(true)).collect();

        let set_b: std::collections::HashSet<_> =
            b.tokens.iter().map(|t| t.normalize(true)).collect();

        let intersection = set_a.intersection(&set_b).count();
        let union = set_a.union(&set_b).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    fn name(&self) -> &str {
        "Jaccard"
    }
}

/// Levenshtein distance-based similarity
pub struct LevenshteinSimilarity;

impl SimilarityMetric for LevenshteinSimilarity {
    fn calculate(&self, a: &TokenSequence, b: &TokenSequence) -> f64 {
        let s1 = a.to_normalized_string(true);
        let s2 = b.to_normalized_string(true);

        let distance = levenshtein_distance(&s1, &s2);
        let max_len = s1.len().max(s2.len());

        if max_len == 0 {
            1.0
        } else {
            1.0 - (distance as f64 / max_len as f64)
        }
    }

    fn name(&self) -> &str {
        "Levenshtein"
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Token, TokenType};

    fn create_test_sequence(tokens: Vec<&str>) -> TokenSequence {
        let tokens = tokens
            .into_iter()
            .enumerate()
            .map(|(i, t)| {
                Token::new(
                    TokenType::Keyword,
                    t.to_string(),
                    i * 4,
                    i * 4 + 3,
                    1,
                    i * 4,
                )
            })
            .collect();
        TokenSequence::new(tokens)
    }

    #[test]
    fn test_jaccard_identical() {
        let seq1 = create_test_sequence(vec!["def", "foo", "bar"]);
        let seq2 = create_test_sequence(vec!["def", "foo", "bar"]);

        let similarity = JaccardSimilarity;
        assert_eq!(similarity.calculate(&seq1, &seq2), 1.0);
    }

    #[test]
    fn test_jaccard_different() {
        let seq1 = create_test_sequence(vec!["def", "foo"]);
        let seq2 = create_test_sequence(vec!["class", "bar"]);

        let similarity = JaccardSimilarity;
        assert_eq!(similarity.calculate(&seq1, &seq2), 0.0);
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }
}
