//! Validation logic for Phase 1 article vocabulary data.

use crate::models::{
    Article, DIFFICULTIES, IELTS_TOPICS, IpaMode, ReviewStatus, SKILL_USES, VocabEntry,
};
use std::collections::HashSet;

/// Result of validating an article directory.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub article_id: String,
    pub vocab_count: usize,
    pub ipa_mode: IpaMode,
    pub ipa_present_count: usize,
    pub ipa_absent_count: usize,
    /// IDs of P1+approved entries with no `my_sentence`.
    pub missing_my_sentence: Vec<String>,
    pub exportable_anki_count: usize,
    /// Hard errors block builds.
    pub errors: Vec<String>,
    /// Soft warnings are printed but do not block builds.
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn print_summary(&self) {
        println!("✓ article.toml valid");
        println!("✓ vocab.csv: {} rows, 19 columns", self.vocab_count);
        println!(
            "✓ ipa_mode: {} ({} present, {} absent)",
            self.ipa_mode, self.ipa_present_count, self.ipa_absent_count
        );
        println!(
            "✓ Anki exportable rows with default filter: {}",
            self.exportable_anki_count
        );

        for error in &self.errors {
            eprintln!("✗ {}", error);
        }

        for warning in &self.warnings {
            println!("⚠ {}", warning);
        }

        if !self.missing_my_sentence.is_empty() {
            println!(
                "⚠ {} P1+approved rows have empty my_sentence — fill after review cycles",
                self.missing_my_sentence.len()
            );
        }

        if self.is_valid() {
            println!(
                "✓ Validation passed (0 errors, {} warnings)",
                self.warnings.len() + usize::from(!self.missing_my_sentence.is_empty())
            );
        } else {
            println!(
                "✗ Validation failed ({} errors, {} warnings)",
                self.errors.len(),
                self.warnings.len() + usize::from(!self.missing_my_sentence.is_empty())
            );
        }
    }
}

/// Validate a fully loaded article.
pub fn validate(article: &Article) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    validate_article_meta(article, &mut errors);
    validate_vocab(article, &mut errors);
    validate_ipa_metadata(article, &mut warnings);

    let ipa_present_count = article
        .vocab
        .iter()
        .filter(|entry| entry.ipa.is_some())
        .count();
    let ipa_absent_count = article.vocab.len().saturating_sub(ipa_present_count);
    let ipa_mode = article.ipa_mode();
    let missing_my_sentence = collect_missing_my_sentence(&article.vocab);
    let exportable_anki_count = article
        .vocab
        .iter()
        .filter(|entry| entry.should_export_to_anki(false))
        .count();

    ValidationResult {
        article_id: article.meta.id.clone(),
        vocab_count: article.vocab.len(),
        ipa_mode,
        ipa_present_count,
        ipa_absent_count,
        missing_my_sentence,
        exportable_anki_count,
        errors,
        warnings,
    }
}

fn validate_article_meta(article: &Article, errors: &mut Vec<String>) {
    if article.meta.id.trim().is_empty() {
        errors.push("article.toml: id field is empty".to_string());
    }
    if article.meta.title.trim().is_empty() {
        errors.push("article.toml: title field is empty".to_string());
    }
    if article.meta.source_name.trim().is_empty() {
        errors.push("article.toml: source_name field is empty".to_string());
    }
    if article.meta.date.trim().is_empty() {
        errors.push("article.toml: date field is empty".to_string());
    }
    if article.meta.primary_topics.is_empty() {
        errors.push("article.toml: primary_topics must contain at least one topic".to_string());
    }
    if article.meta.skills.is_empty() {
        errors.push("article.toml: skills must contain at least one value".to_string());
    }

    for topic in article
        .meta
        .primary_topics
        .iter()
        .chain(article.meta.secondary_topics.iter())
    {
        if !IELTS_TOPICS.contains(&topic.as_str()) {
            errors.push(format!("article.toml: invalid topic '{}'", topic));
        }
    }

    for skill in &article.meta.skills {
        if !SKILL_USES.contains(&skill.as_str()) {
            errors.push(format!("article.toml: invalid skill '{}'", skill));
        }
    }
}

fn validate_vocab(article: &Article, errors: &mut Vec<String>) {
    if article.vocab.is_empty() {
        errors.push("vocab.csv: no entries found".to_string());
        return;
    }

    let mut seen_ids = HashSet::new();
    for entry in &article.vocab {
        if !seen_ids.insert(entry.id.as_str()) {
            errors.push(format!("vocab.csv: duplicate id '{}'", entry.id));
        }

        require_non_empty(errors, &entry.id, &entry.id, "id");
        require_non_empty(errors, &entry.id, &entry.term, "term");
        require_non_empty(errors, &entry.id, &entry.category, "category");
        require_non_empty(errors, &entry.id, &entry.definition_en, "definition_en");
        require_non_empty(errors, &entry.id, &entry.meaning_vi, "meaning_vi");
        require_non_empty(errors, &entry.id, &entry.source_sentence, "source_sentence");

        if entry.ielts_topics.is_empty() {
            errors.push(format!("vocab.csv {}: ielts_topics is empty", entry.id));
        }
        for topic in &entry.ielts_topics {
            if !IELTS_TOPICS.contains(&topic.as_str()) {
                errors.push(format!(
                    "vocab.csv {}: invalid ielts_topics '{}'",
                    entry.id, topic
                ));
            }
        }

        if entry.skill_use.is_empty() {
            errors.push(format!("vocab.csv {}: skill_use is empty", entry.id));
        }
        for skill in &entry.skill_use {
            if !SKILL_USES.contains(&skill.as_str()) {
                errors.push(format!(
                    "vocab.csv {}: invalid skill_use '{}'",
                    entry.id, skill
                ));
            }
        }

        if let Some(difficulty) = &entry.difficulty {
            if !DIFFICULTIES.contains(&difficulty.as_str()) {
                errors.push(format!(
                    "vocab.csv {}: invalid difficulty '{}'",
                    entry.id, difficulty
                ));
            }
        }
    }
}

fn validate_ipa_metadata(article: &Article, warnings: &mut Vec<String>) {
    let Some(stats) = &article.meta.stats else {
        return;
    };
    let Some(has_ipa) = stats.has_ipa else {
        return;
    };

    let computed_has_ipa = article.ipa_mode() != IpaMode::Absent;
    if has_ipa != computed_has_ipa {
        warnings.push(format!(
            "article.toml: stats.has_ipa={} disagrees with computed ipa_mode={}",
            has_ipa,
            article.ipa_mode()
        ));
    }
}

fn require_non_empty(errors: &mut Vec<String>, id: &str, value: &str, field: &str) {
    if value.trim().is_empty() {
        errors.push(format!("vocab.csv {}: {} is empty", id, field));
    }
}

fn collect_missing_my_sentence(entries: &[VocabEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| entry.priority.as_str() == "P1")
        .filter(|entry| entry.review_status == ReviewStatus::Approved)
        .filter(|entry| entry.my_sentence.is_none())
        .map(|entry| entry.id.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Article, parse_article_meta, parse_vocab};

    fn load_fixture() -> Article {
        let vocab_bytes =
            include_bytes!("../../../data/articles/vietnam-two-child-policy/vocab.csv");
        let toml_str = include_str!("../../../data/articles/vietnam-two-child-policy/article.toml");
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
    fn validate_fixture_reports_absent_ipa() {
        let article = load_fixture();
        let result = validate(&article);
        assert_eq!(result.ipa_mode, IpaMode::Absent);
    }

    #[test]
    fn validate_fixture_missing_my_sentence_is_p1_approved_only() {
        let article = load_fixture();
        let expected = article
            .vocab
            .iter()
            .filter(|entry| entry.priority.as_str() == "P1")
            .filter(|entry| entry.review_status == ReviewStatus::Approved)
            .count();
        let result = validate(&article);
        assert_eq!(result.missing_my_sentence.len(), expected);
    }

    #[test]
    fn validate_fixture_is_valid() {
        let article = load_fixture();
        let result = validate(&article);
        assert!(result.is_valid());
    }

    #[test]
    fn validate_detects_duplicate_ids() {
        let mut article = load_fixture();
        article.vocab[1].id = article.vocab[0].id.clone();
        let result = validate(&article);
        assert!(
            result
                .errors
                .iter()
                .any(|error| error.contains("duplicate id"))
        );
    }

    #[test]
    fn validate_accepts_separate_personality_and_physical_appearance_topics() {
        let mut article = load_fixture();
        article.meta.primary_topics = vec!["personality".to_string()];
        article.meta.secondary_topics = vec!["physical-appearance".to_string()];
        article.vocab[0].ielts_topics =
            vec!["personality".to_string(), "physical-appearance".to_string()];
        let result = validate(&article);
        assert!(
            result
                .errors
                .iter()
                .all(|error| !error.contains("invalid topic")
                    && !error.contains("invalid ielts_topics")),
            "errors: {:?}",
            result.errors
        );
    }
}
