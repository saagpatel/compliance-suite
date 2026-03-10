//! Matching Algorithm (Phase 2.4)
//!
//! Token-overlap scoring for questionnaire answer suggestions.
//! Deterministic normalization ensures reproducible scoring.

use crate::answer_bank::AnswerBankEntry;
use crate::domain::errors::{CoreError, CoreErrorCode, CoreResult};
use std::collections::HashSet;

/// A single match suggestion with score and explanation
#[derive(Debug, Clone)]
pub struct MatchSuggestion {
    pub answer_bank_entry_id: String,
    pub score: f64,
    pub question_canonical: String,
    pub answer_short: String,
    pub answer_long: String,
    pub notes: Option<String>,
    pub normalized_question: String,
    pub normalized_answer: String,
    pub confidence_explanation: String,
}

/// Matching engine for questionnaire answer suggestions
pub struct MatchingEngine {
    answer_bank: Vec<AnswerBankEntry>,
}

impl MatchingEngine {
    /// Create a new matching engine with the given answer bank
    pub fn new(answer_bank: Vec<AnswerBankEntry>) -> Self {
        Self { answer_bank }
    }

    /// Normalize text for matching: lowercase, remove punctuation, split into tokens
    ///
    /// Normalization rules:
    /// 1. Convert to lowercase
    /// 2. Remove all punctuation: .,!?;:'"()—-[]{}
    /// 3. Split on whitespace
    /// 4. Filter empty tokens
    /// 5. Sort tokens for deterministic comparison
    pub fn normalize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .chars()
            .map(|c| {
                // Remove punctuation
                if ".,!?;:'\"()—-[]{}".contains(c) {
                    ' '
                } else {
                    c
                }
            })
            .collect::<String>()
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    }

    /// Calculate token overlap score between question and answer
    ///
    /// Score formula: (intersection count) / (union count)
    /// Range: 0.0 (no overlap) to 1.0 (identical token sets)
    fn score_tokens(q_tokens: &[String], a_tokens: &[String]) -> f64 {
        if q_tokens.is_empty() && a_tokens.is_empty() {
            return 0.0;
        }

        let q_set: HashSet<&String> = q_tokens.iter().collect();
        let a_set: HashSet<&String> = a_tokens.iter().collect();

        let intersection_count = q_set.intersection(&a_set).count();
        let union_count = q_set.union(&a_set).count();

        if union_count == 0 {
            return 0.0;
        }

        intersection_count as f64 / union_count as f64
    }

    /// Generate confidence explanation from token overlap
    fn explain_confidence(score: f64, q_tokens: &[String], a_tokens: &[String]) -> String {
        let q_set: HashSet<&String> = q_tokens.iter().collect();
        let a_set: HashSet<&String> = a_tokens.iter().collect();

        let common_tokens: Vec<String> = q_set
            .intersection(&a_set)
            .take(5) // Show up to 5 common tokens
            .map(|s| format!("'{}'", s))
            .collect();

        let percentage = (score * 100.0) as u32;

        if common_tokens.is_empty() {
            format!("{}% match: no common tokens", percentage)
        } else {
            format!(
                "{}% token overlap: {}",
                percentage,
                common_tokens.join(", ")
            )
        }
    }

    /// Get top-N matching suggestions for a question
    ///
    /// Returns suggestions sorted by score (highest first), up to `top_n` results.
    /// Only returns suggestions with score > 0.0
    pub fn get_suggestions(
        &self,
        question: &str,
        top_n: usize,
    ) -> CoreResult<Vec<MatchSuggestion>> {
        if question.trim().is_empty() {
            return Err(CoreError {
                code: CoreErrorCode::ValidationError,
                message: "Question cannot be empty".to_string(),
            });
        }

        if self.answer_bank.is_empty() {
            return Ok(Vec::new());
        }

        let q_tokens = Self::normalize(question);

        let mut suggestions: Vec<MatchSuggestion> = self
            .answer_bank
            .iter()
            .map(|entry| {
                // Combine question_canonical + answer_short + answer_long for matching
                let combined_answer = format!(
                    "{} {} {}",
                    entry.question_canonical, entry.answer_short, entry.answer_long
                );
                let a_tokens = Self::normalize(&combined_answer);

                let score = Self::score_tokens(&q_tokens, &a_tokens);
                let confidence_explanation = Self::explain_confidence(score, &q_tokens, &a_tokens);

                MatchSuggestion {
                    answer_bank_entry_id: entry.entry_id.clone(),
                    score,
                    question_canonical: entry.question_canonical.clone(),
                    answer_short: entry.answer_short.clone(),
                    answer_long: entry.answer_long.clone(),
                    notes: entry.notes.clone(),
                    normalized_question: q_tokens.join(" "),
                    normalized_answer: a_tokens.join(" "),
                    confidence_explanation,
                }
            })
            .filter(|s| s.score > 0.0) // Only non-zero scores
            .collect();

        // Sort by score descending
        suggestions.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take top N
        suggestions.truncate(top_n);

        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_removes_punctuation() {
        let result = MatchingEngine::normalize("Hello, World! How are you?");
        assert_eq!(result, vec!["hello", "world", "how", "are", "you"]);
    }

    #[test]
    fn test_normalize_handles_unicode() {
        let result = MatchingEngine::normalize("Café résumé");
        assert_eq!(result, vec!["café", "résumé"]);
    }

    #[test]
    fn test_score_tokens_identical() {
        let tokens_a = vec!["access".to_string(), "control".to_string()];
        let tokens_b = vec!["access".to_string(), "control".to_string()];
        let score = MatchingEngine::score_tokens(&tokens_a, &tokens_b);
        assert!((score - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_score_tokens_partial_overlap() {
        let tokens_a = vec!["access".to_string(), "control".to_string()];
        let tokens_b = vec![
            "access".to_string(),
            "control".to_string(),
            "rbac".to_string(),
        ];
        let score = MatchingEngine::score_tokens(&tokens_a, &tokens_b);
        // 2 common / 3 total = 0.67
        assert!((score - 0.6666).abs() < 0.01);
    }

    #[test]
    fn test_score_tokens_no_overlap() {
        let tokens_a = vec!["access".to_string(), "control".to_string()];
        let tokens_b = vec!["network".to_string(), "firewall".to_string()];
        let score = MatchingEngine::score_tokens(&tokens_a, &tokens_b);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_empty_answer_bank_returns_empty() {
        let engine = MatchingEngine::new(Vec::new());
        let result = engine.get_suggestions("test question", 3).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_empty_question_returns_error() {
        let engine = MatchingEngine::new(Vec::new());
        let result = engine.get_suggestions("", 3);
        assert!(result.is_err());
    }
}
