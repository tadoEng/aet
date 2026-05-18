//! Validation logic for article content.
//!
//! Validation is split into hard errors (block the build) and soft warnings
//! (printed but do not prevent output). This matches the product decision:
//! missing `my_sentence` is a warning, not an error.

use crate::models::{Article, VocabEntry};

/// Result of validating an article directory.
#[derive(Debug)]
pub struct ValidationResult {
    pub vocab_count: usize,
    /// IDs of entries with no `my_sentence`.
    pub missing_my_sentence: Vec<String>,
    /// Hard errors — these block the build.
    pub errors: Vec<String>,
    /// Soft warnings — printed but do not block.
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Returns true if there are no hard errors.
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Print a human-readable summary to stdout.
    pub fn print_summary(&self, article_id: &str) {
        println!("✓ article.toml — valid");
        println!("✓ vocab.csv — {} entries loaded", self.vocab_count);

        for error in &self.errors {
            eprintln!("✗ {}", error);
        }

        for warning in &self.warnings {
            println!("⚠ {}", warning);
        }

        if !self.missing_my_sentence.is_empty() {
            println!(
                "⚠ {} entries missing my_sentence — add production practice sentences for best results",
                self.missing_my_sentence.len()
            );
        }

        if self.is_valid() {
            println!("✓ {} — ready to build", article_id);
        }
    }
}

/// Validate a fully loaded article.
pub fn validate(article: &Article) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Hard checks
    if article.meta.id.is_empty() {
        errors.push("article.toml: id field is empty".to_string());
    }
    if article.meta.title.is_empty() {
        errors.push("article.toml: title field is empty".to_string());
    }
    if article.vocab.is_empty() {
        errors.push("vocab.csv: no entries found".to_string());
    }

    // Check for duplicate IDs
    let mut seen_ids = std::collections::HashSet::new();
    for entry in &article.vocab {
        if !seen_ids.insert(&entry.id) {
            errors.push(format!("vocab.csv: duplicate id '{}'", entry.id));
        }
    }

    // Check cloze-able entries have a source sentence
    let mut cloze_warnings = Vec::new();
    for entry in &article.vocab {
        if entry.card_types.contains(&"cloze".to_string())
            && entry.source_sentence.trim().is_empty()
        {
            cloze_warnings.push(entry.id.clone());
        }
    }
    if !cloze_warnings.is_empty() {
        warnings.push(format!(
            "{} cloze entries have empty source_sentence — they will be skipped during Anki export: {}",
            cloze_warnings.len(),
            cloze_warnings.join(", ")
        ));
    }

    let missing_my_sentence = collect_missing_my_sentence(&article.vocab);

    ValidationResult {
        vocab_count: article.vocab.len(),
        missing_my_sentence,
        errors,
        warnings,
    }
}

fn collect_missing_my_sentence(entries: &[VocabEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|e| e.my_sentence.is_none())
        .map(|e| e.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{parse_article_meta, parse_vocab, Article};

    fn load_fixture() -> Article {
        let vocab_bytes =
            include_bytes!("../../../data/articles/vietnam-two-child-policy/vocab.csv");
        let toml_str =
            include_str!("../../../data/articles/vietnam-two-child-policy/article.toml");
        Article {
            meta: parse_article_meta(toml_str).unwrap(),
            vocab: parse_vocab(vocab_bytes).unwrap(),
        }
    }

    #[test]
    fn validate_fixture_should_have_no_hard_errors() {
        let article = load_fixture();
        let result = validate(&article);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    }

    #[test]
    fn validate_fixture_all_entries_missing_my_sentence() {
        let article = load_fixture();
        let result = validate(&article);
        assert_eq!(result.missing_my_sentence.len(), 88);
    }

    #[test]
    fn validate_fixture_is_valid() {
        let article = load_fixture();
        let result = validate(&article);
        assert!(result.is_valid());
    }
}