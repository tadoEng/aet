//! Data models for AET content types.
//!
//! These structs are the single source of truth for how vocabulary entries
//! and article metadata are represented in memory. All parsers produce these
//! types; all exporters consume them.

use serde::Deserialize;

/// A single vocabulary entry parsed from vocab.csv.
///
/// The `id` field uses topic-prefix + sequential format (e.g. `pop-001`).
/// Once assigned and used to generate an Anki card, an ID is frozen.
#[derive(Debug, Clone, Deserialize)]
pub struct VocabEntry {
    pub id: String,
    pub source_type: String,
    pub source_id: String,
    pub term: String,
    pub category: String,
    /// Grammatical type — stored as `type` in CSV, renamed here to avoid keyword clash.
    #[serde(rename = "type")]
    pub entry_type: String,
    pub definition_en: String,
    pub meaning_vi: String,
    pub source_sentence: String,
    /// Empty string in CSV becomes None here.
    pub my_sentence: Option<String>,
    pub collocation_pattern: String,
    pub ielts_topic: String,
    /// Semicolon-separated in CSV, split into Vec on parse.
    pub skill_use: Vec<String>,
    pub difficulty: String,
    /// Semicolon-separated in CSV, split into Vec on parse.
    pub card_types: Vec<String>,
    pub status: String,
    /// Space-separated in CSV, split into Vec on parse.
    pub tags: Vec<String>,
}

/// Raw row from CSV before post-processing (splitting, Option conversion).
#[derive(Debug, Deserialize)]
struct RawVocabRow {
    id: String,
    source_type: String,
    source_id: String,
    term: String,
    category: String,
    #[serde(rename = "type")]
    entry_type: String,
    definition_en: String,
    meaning_vi: String,
    source_sentence: String,
    my_sentence: String,
    collocation_pattern: String,
    ielts_topic: String,
    skill_use: String,
    difficulty: String,
    card_types: String,
    status: String,
    tags: String,
}

impl From<RawVocabRow> for VocabEntry {
    fn from(raw: RawVocabRow) -> Self {
        Self {
            id: raw.id,
            source_type: raw.source_type,
            source_id: raw.source_id,
            term: raw.term,
            category: raw.category,
            entry_type: raw.entry_type,
            definition_en: raw.definition_en,
            meaning_vi: raw.meaning_vi,
            source_sentence: raw.source_sentence,
            my_sentence: if raw.my_sentence.trim().is_empty() {
                None
            } else {
                Some(raw.my_sentence)
            },
            collocation_pattern: raw.collocation_pattern,
            ielts_topic: raw.ielts_topic,
            skill_use: split_semicolon(&raw.skill_use),
            difficulty: raw.difficulty,
            card_types: split_semicolon(&raw.card_types),
            status: raw.status,
            tags: raw.tags.split_whitespace().map(str::to_owned).collect(),
        }
    }
}

fn split_semicolon(s: &str) -> Vec<String> {
    s.split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Article metadata parsed from article.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct ArticleMeta {
    pub id: String,
    pub title: String,
    pub source: String,
    pub author: String,
    pub date: String,
    pub level: String,
    pub topics: Vec<String>,
    pub skills: Vec<String>,
    pub subtitle: Option<String>,
    pub contributor: Option<String>,
}

/// A fully loaded article — metadata + all vocabulary entries.
#[derive(Debug)]
pub struct Article {
    pub meta: ArticleMeta,
    pub vocab: Vec<VocabEntry>,
}

impl Article {
    /// Returns entries grouped by category, preserving first-appearance order.
    pub fn vocab_by_category(&self) -> Vec<(&str, Vec<&VocabEntry>)> {
        let mut seen: Vec<&str> = Vec::new();
        for entry in &self.vocab {
            if !seen.contains(&entry.category.as_str()) {
                seen.push(&entry.category);
            }
        }
        seen.into_iter()
            .map(|cat| {
                let entries = self
                    .vocab
                    .iter()
                    .filter(|e| e.category == cat)
                    .collect::<Vec<_>>();
                (cat, entries)
            })
            .collect()
    }
}

/// Parse vocab.csv from a byte slice. Returns all entries.
pub fn parse_vocab(data: &[u8]) -> anyhow::Result<Vec<VocabEntry>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(data);

    let mut entries = Vec::new();
    for (i, result) in reader.deserialize::<RawVocabRow>().enumerate() {
        let raw = result.map_err(|e| anyhow::anyhow!("CSV row {}: {}", i + 2, e))?;
        entries.push(VocabEntry::from(raw));
    }
    Ok(entries)
}

/// Parse article.toml from a string slice.
pub fn parse_article_meta(content: &str) -> anyhow::Result<ArticleMeta> {
    toml::from_str(content).map_err(|e| anyhow::anyhow!("Failed to parse article.toml: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] = include_bytes!("../../../data/articles/vietnam-two-child-policy/vocab.csv");
    const FIXTURE_TOML: &str =
        include_str!("../../../data/articles/vietnam-two-child-policy/article.toml");

    #[test]
    fn parse_vocab_should_load_all_88_entries() {
        let entries = parse_vocab(FIXTURE).expect("should parse");
        assert_eq!(entries.len(), 88);
    }

    #[test]
    fn parse_vocab_first_entry_should_be_demographic_shift() {
        let entries = parse_vocab(FIXTURE).expect("should parse");
        assert_eq!(entries[0].id, "pop-001");
        assert_eq!(entries[0].term, "demographic shift");
    }

    #[test]
    fn parse_vocab_my_sentence_empty_becomes_none() {
        let entries = parse_vocab(FIXTURE).expect("should parse");
        // All entries in fixture have empty my_sentence
        assert!(
            entries.iter().all(|e| e.my_sentence.is_none()),
            "all fixture entries should have None my_sentence"
        );
    }

    #[test]
    fn parse_vocab_skill_use_splits_on_semicolon() {
        let entries = parse_vocab(FIXTURE).expect("should parse");
        let pop001 = entries.iter().find(|e| e.id == "pop-001").unwrap();
        assert!(pop001.skill_use.contains(&"writing-task2".to_string()));
        assert!(pop001.skill_use.contains(&"speaking-part3".to_string()));
    }

    #[test]
    fn parse_article_meta_should_load_title() {
        let meta = parse_article_meta(FIXTURE_TOML).expect("should parse");
        assert_eq!(meta.id, "vietnam-two-child-policy");
        assert!(meta.title.contains("Vietnam"));
    }
}