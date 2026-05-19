//! Data models for AET article vocabulary content.

use anyhow::{Context, bail};
use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

pub const VOCAB_HEADERS: [&str; 19] = [
    "id",
    "term",
    "ipa",
    "category",
    "type",
    "definition_en",
    "meaning_vi",
    "source_sentence",
    "my_sentence",
    "priority",
    "collocation_pattern",
    "ielts_topics",
    "skill_use",
    "difficulty",
    "card_types",
    "origin",
    "review_status",
    "anki_export",
    "tags",
];

pub const ENTRY_TYPES: [&str; 8] = [
    "word",
    "noun_phrase",
    "verb_phrase",
    "collocation",
    "fixed_phrase",
    "prepositional_pattern",
    "reporting_verb",
    "sentence_frame",
];

pub const IELTS_TOPICS: [&str; 12] = [
    "environment",
    "poverty-global-issues",
    "law-crimes",
    "business-economics",
    "health-medicine",
    "education",
    "family-relationships-teenagers",
    "urban-rural",
    "personality",
    "physical-appearance",
    "work-jobs",
    "government-policy",
];

pub const SKILL_USES: [&str; 6] = [
    "reading",
    "writing-task1",
    "writing-task2",
    "speaking-part2",
    "speaking-part3",
    "listening",
];

pub const DIFFICULTIES: [&str; 4] = ["B1", "B2", "C1", "C2"];

/// A single vocabulary entry parsed from the locked Phase 1 vocab.csv schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabEntry {
    pub id: String,
    pub term: String,
    pub ipa: Option<String>,
    pub category: String,
    pub entry_type: EntryType,
    pub definition_en: String,
    pub meaning_vi: String,
    pub source_sentence: String,
    pub my_sentence: Option<String>,
    pub priority: Priority,
    pub collocation_pattern: Option<String>,
    pub ielts_topics: Vec<String>,
    pub skill_use: Vec<String>,
    pub difficulty: Option<String>,
    pub card_types: Vec<CardType>,
    pub origin: Origin,
    pub review_status: ReviewStatus,
    pub anki_export: bool,
    pub tags: Vec<String>,
}

impl VocabEntry {
    pub fn should_export_to_anki(&self, all_priorities: bool) -> bool {
        (all_priorities || self.priority == Priority::P1)
            && self.review_status == ReviewStatus::Approved
            && self.anki_export
            && !self.source_sentence.trim().is_empty()
    }
}

/// Raw row from CSV before conversion into typed values.
#[derive(Debug, Deserialize)]
struct RawVocabRow {
    id: String,
    term: String,
    ipa: String,
    category: String,
    #[serde(rename = "type")]
    entry_type: String,
    definition_en: String,
    meaning_vi: String,
    source_sentence: String,
    my_sentence: String,
    priority: String,
    collocation_pattern: String,
    ielts_topics: String,
    skill_use: String,
    difficulty: String,
    card_types: String,
    origin: String,
    review_status: String,
    anki_export: String,
    tags: String,
}

impl TryFrom<RawVocabRow> for VocabEntry {
    type Error = anyhow::Error;

    fn try_from(raw: RawVocabRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: raw.id,
            term: raw.term,
            ipa: optional_string(raw.ipa),
            category: raw.category,
            entry_type: raw.entry_type.parse()?,
            definition_en: raw.definition_en,
            meaning_vi: raw.meaning_vi,
            source_sentence: raw.source_sentence,
            my_sentence: optional_string(raw.my_sentence),
            priority: raw.priority.parse()?,
            collocation_pattern: optional_string(raw.collocation_pattern),
            ielts_topics: split_semicolon(&raw.ielts_topics),
            skill_use: split_semicolon(&raw.skill_use),
            difficulty: optional_string(raw.difficulty),
            card_types: parse_semicolon_values(&raw.card_types, "card_types")?,
            origin: raw.origin.parse()?,
            review_status: raw.review_status.parse()?,
            anki_export: parse_yes_no(&raw.anki_export, "anki_export")?,
            tags: split_semicolon(&raw.tags),
        })
    }
}

/// Article metadata parsed from article.toml.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ArticleMeta {
    pub id: String,
    pub title: String,
    pub source_name: String,
    #[serde(default)]
    pub author: Option<String>,
    pub date: String,
    pub primary_topics: Vec<String>,
    #[serde(default)]
    pub secondary_topics: Vec<String>,
    pub skills: Vec<String>,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub teacher_pdf: Option<String>,
    #[serde(default)]
    pub private: bool,
    #[serde(default)]
    pub copyright_mode: Option<String>,
    #[serde(default)]
    pub stats: Option<ArticleStats>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ArticleStats {
    #[serde(default)]
    pub has_ipa: Option<bool>,
}

/// A fully loaded article: metadata plus all vocabulary entries.
#[derive(Debug, Clone)]
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
            .map(|category| {
                let entries = self
                    .vocab
                    .iter()
                    .filter(|entry| entry.category == category)
                    .collect();
                (category, entries)
            })
            .collect()
    }

    pub fn ipa_mode(&self) -> IpaMode {
        IpaMode::from_entries(&self.vocab)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpaMode {
    Absent,
    Mixed,
    Present,
}

impl IpaMode {
    pub fn from_entries(entries: &[VocabEntry]) -> Self {
        let present = entries.iter().filter(|entry| entry.ipa.is_some()).count();
        match (present, entries.len()) {
            (0, _) => Self::Absent,
            (present, total) if present == total => Self::Present,
            _ => Self::Mixed,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Mixed => "mixed",
            Self::Present => "present",
        }
    }

    pub fn has_column(self) -> bool {
        matches!(self, Self::Mixed | Self::Present)
    }
}

impl fmt::Display for IpaMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

macro_rules! phase1_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.trim() {
                    $($value => Ok(Self::$variant),)+
                    other => bail!("invalid {} '{}'", stringify!($name), other),
                }
            }
        }
    };
}

phase1_enum!(EntryType {
    Word => "word",
    NounPhrase => "noun_phrase",
    VerbPhrase => "verb_phrase",
    Collocation => "collocation",
    FixedPhrase => "fixed_phrase",
    PrepositionalPattern => "prepositional_pattern",
    ReportingVerb => "reporting_verb",
    SentenceFrame => "sentence_frame",
});

phase1_enum!(Priority {
    P1 => "P1",
    P2 => "P2",
    P3 => "P3",
});

phase1_enum!(CardType {
    Basic => "basic",
    Cloze => "cloze",
    Production => "production",
});

phase1_enum!(Origin {
    Teacher => "teacher",
    Codex => "codex",
    User => "user",
    Manual => "manual",
});

phase1_enum!(ReviewStatus {
    Draft => "draft",
    Reviewed => "reviewed",
    Approved => "approved",
    Rejected => "rejected",
});

/// Parse vocab.csv from a byte slice and enforce the locked 19-column header.
pub fn parse_vocab(data: &[u8]) -> anyhow::Result<Vec<VocabEntry>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(data);

    let headers = reader.headers().context("vocab.csv: missing header row")?;
    let actual_headers = headers.iter().collect::<Vec<_>>();
    if actual_headers != VOCAB_HEADERS {
        bail!(
            "vocab.csv: expected columns '{}', got '{}'",
            VOCAB_HEADERS.join(","),
            actual_headers.join(",")
        );
    }

    let mut entries = Vec::new();
    for (index, result) in reader.deserialize::<RawVocabRow>().enumerate() {
        let raw = result.map_err(|error| anyhow::anyhow!("CSV row {}: {}", index + 2, error))?;
        let entry = VocabEntry::try_from(raw)
            .map_err(|error| anyhow::anyhow!("CSV row {}: {}", index + 2, error))?;
        entries.push(entry);
    }
    Ok(entries)
}

/// Parse article.toml from a string slice.
pub fn parse_article_meta(content: &str) -> anyhow::Result<ArticleMeta> {
    toml::from_str(content).map_err(|error| anyhow::anyhow!("article.toml: {}", error))
}

fn split_semicolon(s: &str) -> Vec<String> {
    s.split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

fn parse_semicolon_values<T>(s: &str, field: &str) -> anyhow::Result<Vec<T>>
where
    T: FromStr<Err = anyhow::Error>,
{
    let values = split_semicolon(s);
    if values.is_empty() {
        bail!("{} must contain at least one value", field);
    }
    values
        .into_iter()
        .map(|value| {
            value
                .parse()
                .map_err(|error| anyhow::anyhow!("{}: {}", field, error))
        })
        .collect()
}

fn optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_yes_no(value: &str, field: &str) -> anyhow::Result<bool> {
    match value.trim() {
        "yes" => Ok(true),
        "no" => Ok(false),
        other => bail!("{} must be 'yes' or 'no', got '{}'", field, other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &[u8] =
        include_bytes!("../../../data/articles/vietnam-two-child-policy/vocab.csv");
    const FIXTURE_TOML: &str =
        include_str!("../../../data/articles/vietnam-two-child-policy/article.toml");

    fn sample_entry(ipa: Option<&str>) -> VocabEntry {
        VocabEntry {
            id: "x-001".to_string(),
            term: "test term".to_string(),
            ipa: ipa.map(str::to_string),
            category: "Category".to_string(),
            entry_type: EntryType::Collocation,
            definition_en: "definition".to_string(),
            meaning_vi: "nghia".to_string(),
            source_sentence: "A test term appears here.".to_string(),
            my_sentence: None,
            priority: Priority::P1,
            collocation_pattern: Some("test [term]".to_string()),
            ielts_topics: vec!["education".to_string()],
            skill_use: vec!["writing-task2".to_string()],
            difficulty: Some("B2".to_string()),
            card_types: vec![CardType::Basic, CardType::Cloze, CardType::Production],
            origin: Origin::Teacher,
            review_status: ReviewStatus::Approved,
            anki_export: true,
            tags: vec!["test".to_string()],
        }
    }

    #[test]
    fn parse_vocab_should_load_vietnam_fixture_entries() {
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
    fn parse_vocab_rejects_wrong_column_order() {
        let csv = b"id,term\nx-001,test\n";
        let error = parse_vocab(csv).unwrap_err().to_string();
        assert!(error.contains("expected columns"));
    }

    #[test]
    fn parse_vocab_rejects_invalid_controlled_value() {
        let csv = format!(
            "{}\nx-001,test,,Category,bad_type,definition,meaning,source,,P1,pattern,education,writing-task2,B2,basic,teacher,approved,yes,tag\n",
            VOCAB_HEADERS.join(",")
        );
        let error = parse_vocab(csv.as_bytes()).unwrap_err().to_string();
        assert!(error.contains("invalid EntryType"));
    }

    #[test]
    fn parse_vocab_invalid_card_types_error_names_field() {
        let csv = format!(
            "{}\nx-001,test,,Category,word,definition,meaning,source,,P1,pattern,education,writing-task2,B2,quiz,teacher,approved,yes,tag\n",
            VOCAB_HEADERS.join(",")
        );
        let error = parse_vocab(csv.as_bytes()).unwrap_err().to_string();
        assert!(error.contains("card_types"));
        assert!(error.contains("invalid CardType"));
    }

    #[test]
    fn parse_vocab_my_sentence_empty_becomes_none() {
        let entries = parse_vocab(FIXTURE).expect("should parse");
        assert!(entries.iter().all(|entry| entry.my_sentence.is_none()));
    }

    #[test]
    fn parse_vocab_skill_use_splits_on_semicolon() {
        let entries = parse_vocab(FIXTURE).expect("should parse");
        let pop001 = entries.iter().find(|entry| entry.id == "pop-001").unwrap();
        assert!(pop001.skill_use.contains(&"writing-task2".to_string()));
        assert!(pop001.skill_use.contains(&"speaking-part3".to_string()));
    }

    #[test]
    fn parse_article_meta_should_load_title() {
        let meta = parse_article_meta(FIXTURE_TOML).expect("should parse");
        assert_eq!(meta.id, "vietnam-two-child-policy");
        assert!(meta.title.contains("Vietnam"));
    }

    #[test]
    fn ipa_mode_detects_absent_mixed_and_present() {
        assert_eq!(
            IpaMode::from_entries(&[sample_entry(None)]),
            IpaMode::Absent
        );
        assert_eq!(
            IpaMode::from_entries(&[sample_entry(Some("/test/")), sample_entry(None)]),
            IpaMode::Mixed
        );
        assert_eq!(
            IpaMode::from_entries(&[sample_entry(Some("/test/"))]),
            IpaMode::Present
        );
    }
}
