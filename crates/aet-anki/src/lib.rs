//! Anki TSV exporters for AET vocabulary entries.
//!
//! Produces two TSV files importable into Anki desktop:
//! - `anki-basic.tsv`  — front/back question cards
//! - `anki-cloze.tsv`  — sentence-based cloze deletion cards
//!
//! Both files are UTF-8, LF line endings, tab-delimited, no header row.

use aet_core::{Article, VocabEntry};
use anyhow::Result;

/// Result of an Anki export operation.
#[derive(Debug)]
pub struct AnkiExportResult {
    pub basic_count: usize,
    pub cloze_count: usize,
    /// IDs skipped during cloze generation (term not found in sentence).
    pub cloze_skipped: Vec<String>,
    /// Note that reverse cards are deferred to v0.2.
    pub reverse_notice: bool,
}

/// Generate content for `anki-basic.tsv`.
///
/// Columns: `front\tback\ttags`
/// Only entries with `"basic"` in card_types are included.
pub fn generate_basic_tsv(article: &Article) -> Result<(String, usize)> {
    let mut rows: Vec<String> = Vec::new();

    for entry in &article.vocab {
        if !entry.card_types.contains(&"basic".to_string()) {
            continue;
        }

        let front = format!("What does \"{}\" mean?", entry.term);
        let back = build_basic_back(entry);
        let tags = build_tags(entry, &article.meta.id);

        rows.push(format!("{}\t{}\t{}", front, back, tags));
    }

    let count = rows.len();
    Ok((rows.join("\n"), count))
}

/// Generate content for `anki-cloze.tsv`.
///
/// Columns: `text\tback_extra\ttags`
/// Only entries with `"cloze"` in card_types are included.
/// Returns the TSV content, entry count, and list of skipped IDs.
pub fn generate_cloze_tsv(article: &Article) -> Result<(String, usize, Vec<String>)> {
    let mut rows: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();

    for entry in &article.vocab {
        if !entry.card_types.contains(&"cloze".to_string()) {
            continue;
        }

        match build_cloze_text(entry) {
            Some(text) => {
                let back_extra = format!(
                    "{} | Vietnamese: {}",
                    entry.definition_en, entry.meaning_vi
                );
                let tags = build_tags(entry, &article.meta.id);
                rows.push(format!("{}\t{}\t{}", text, back_extra, tags));
            }
            None => {
                eprintln!(
                    "⚠ Could not generate cloze for {}: term not found in source_sentence or collocation_pattern",
                    entry.id
                );
                skipped.push(entry.id.clone());
            }
        }
    }

    let count = rows.len();
    Ok((rows.join("\n"), count, skipped))
}

/// Build the back of a basic card.
fn build_basic_back(entry: &VocabEntry) -> String {
    let mut parts = vec![
        entry.definition_en.clone(),
        format!("Vietnamese: {}", entry.meaning_vi),
    ];

    if !entry.source_sentence.is_empty() {
        parts.push(format!("Example: {}", entry.source_sentence));
    }

    if let Some(ref my) = entry.my_sentence {
        parts.push(format!("Your sentence: {}", my));
    }

    // Use <br> for Anki HTML rendering in the back field
    parts.join("<br><br>")
}

/// Try to build a cloze deletion text.
///
/// Strategy:
/// 1. Try to find the term (case-insensitive) in the source_sentence.
/// 2. Fall back to the collocation_pattern.
/// 3. If neither works, return None (entry will be skipped).
fn build_cloze_text(entry: &VocabEntry) -> Option<String> {
    // Try source_sentence first
    if !entry.source_sentence.is_empty() {
        if let Some(cloze) = replace_term_with_cloze(&entry.source_sentence, &entry.term) {
            return Some(cloze);
        }
    }

    // Fall back to collocation_pattern as the sentence base
    if !entry.collocation_pattern.is_empty() {
        if let Some(cloze) =
            replace_term_with_cloze(&entry.collocation_pattern, &entry.term)
        {
            return Some(cloze);
        }
    }

    None
}

/// Replace the first occurrence of `term` in `sentence` (case-insensitive)
/// with `{{c1::term}}` using the original casing from the sentence.
fn replace_term_with_cloze(sentence: &str, term: &str) -> Option<String> {
    let lower_sentence = sentence.to_lowercase();
    let lower_term = term.to_lowercase();

    lower_sentence.find(&lower_term).map(|start| {
        let end = start + lower_term.len();
        let original_slice = &sentence[start..end];
        format!(
            "{}{{{{c1::{}}}}}{}",
            &sentence[..start],
            original_slice,
            &sentence[end..]
        )
    })
}

/// Build the tags string for an Anki row.
///
/// Format: space-separated, lowercase.
/// Always includes: `aet`, the article id (slug), the ielts_topic, the difficulty.
fn build_tags(entry: &VocabEntry, article_id: &str) -> String {
    let mut tags: Vec<String> = vec![
        "aet".to_string(),
        article_id.to_string(),
        entry.ielts_topic.to_lowercase(),
        entry.difficulty.to_lowercase(),
    ];
    for tag in &entry.tags {
        let t = tag.to_lowercase();
        if !tags.contains(&t) {
            tags.push(t);
        }
    }
    tags.join(" ")
}

/// Write both TSV files to the output directory and return a summary.
pub fn export(article: &Article, out_dir: &std::path::Path) -> Result<AnkiExportResult> {
    std::fs::create_dir_all(out_dir)?;

    // Check for reverse cards in the data (deferred to v0.2)
    let has_reverse = article
        .vocab
        .iter()
        .any(|e| e.card_types.contains(&"reverse".to_string()));

    let (basic_tsv, basic_count) = generate_basic_tsv(article)?;
    let basic_path = out_dir.join("anki-basic.tsv");
    std::fs::write(&basic_path, basic_tsv.as_bytes())?;

    let (cloze_tsv, cloze_count, cloze_skipped) = generate_cloze_tsv(article)?;
    let cloze_path = out_dir.join("anki-cloze.tsv");
    std::fs::write(&cloze_path, cloze_tsv.as_bytes())?;

    Ok(AnkiExportResult {
        basic_count,
        cloze_count,
        cloze_skipped,
        reverse_notice: has_reverse,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use aet_core::models::{parse_article_meta, parse_vocab, Article};

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
    fn generate_basic_tsv_should_produce_rows() {
        let article = load_fixture();
        let (tsv, count) = generate_basic_tsv(&article).unwrap();
        assert!(count > 0, "should produce at least one basic card");
        assert!(tsv.contains("What does"), "should contain question prefix");
    }

    #[test]
    fn generate_basic_tsv_rows_are_tab_separated() {
        let article = load_fixture();
        let (tsv, _) = generate_basic_tsv(&article).unwrap();
        let first_line = tsv.lines().next().unwrap();
        let parts: Vec<&str> = first_line.split('\t').collect();
        assert_eq!(parts.len(), 3, "each row must have exactly 3 tab-separated fields");
    }

    #[test]
    fn generate_cloze_tsv_should_produce_rows() {
        let article = load_fixture();
        let (tsv, count, _skipped) = generate_cloze_tsv(&article).unwrap();
        assert!(count > 0, "should produce at least one cloze card");
        assert!(tsv.contains("{{c1::"), "should contain cloze syntax");
    }

    #[test]
    fn replace_term_with_cloze_finds_term_in_sentence() {
        let result = replace_term_with_cloze(
            "Vietnam is facing a demographic shift as its population ages.",
            "demographic shift",
        );
        assert!(result.is_some());
        assert!(result.unwrap().contains("{{c1::demographic shift}}"));
    }

    #[test]
    fn replace_term_with_cloze_returns_none_when_not_found() {
        let result = replace_term_with_cloze("This sentence has nothing.", "missing term");
        assert!(result.is_none());
    }

    #[test]
    fn build_tags_includes_required_fields() {
        let article = load_fixture();
        let entry = &article.vocab[0];
        let tags = build_tags(entry, &article.meta.id);
        assert!(tags.contains("aet"));
        assert!(tags.contains("vietnam-two-child-policy"));
        assert!(tags.contains("b2"));
    }
}