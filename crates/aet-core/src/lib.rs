//! AET Core — data models, parsing, and validation.
//!
//! This crate is the foundation. All other AET crates depend on it.
//! It has no knowledge of output formats (Anki, Typst, JSON).

pub mod models;
pub mod validator;

pub use models::{
    Article, ArticleMeta, CardType, EntryType, IELTS_TOPICS, IpaMode, Origin, Priority,
    ReviewStatus, VocabEntry, parse_article_meta, parse_vocab,
};
pub use validator::{ValidationResult, validate};

use anyhow::{Context, Result};
use std::path::Path;

/// Load a complete article from a directory path.
///
/// Expects:
/// - `{dir}/article.toml`
/// - `{dir}/vocab.csv`
pub fn load_article(dir: &Path) -> Result<Article> {
    let toml_path = dir.join("article.toml");
    let csv_path = dir.join("vocab.csv");

    let toml_content = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("Cannot read {}", toml_path.display()))?;

    let csv_bytes =
        std::fs::read(&csv_path).with_context(|| format!("Cannot read {}", csv_path.display()))?;

    let meta = parse_article_meta(&toml_content)?;
    let vocab = parse_vocab(&csv_bytes)?;

    Ok(Article { meta, vocab })
}

/// Load every article folder under `data/articles`.
pub fn load_articles(root: &Path) -> Result<Vec<Article>> {
    let mut articles = Vec::new();
    for entry in
        std::fs::read_dir(root).with_context(|| format!("Cannot read {}", root.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        articles
            .push(load_article(&path).with_context(|| format!("Cannot load {}", path.display()))?);
    }
    articles.sort_by(|left, right| left.meta.id.cmp(&right.meta.id));
    Ok(articles)
}

/// Build a synthetic article containing every entry tagged with a canonical IELTS topic.
pub fn build_topic_article(topic: &str, articles: &[Article]) -> Article {
    let mut vocab = Vec::new();
    for article in articles {
        for entry in &article.vocab {
            if entry
                .ielts_topics
                .iter()
                .any(|entry_topic| entry_topic == topic)
            {
                let mut cloned = entry.clone();
                cloned.category = format!("{} / {}", article.meta.title, entry.category);
                vocab.push(cloned);
            }
        }
    }

    Article {
        meta: ArticleMeta {
            id: format!("topic-{}", topic),
            title: format!("Topic Bank: {}", topic),
            source_name: "AET Topic Bank".to_string(),
            author: None,
            date: "cross-article".to_string(),
            primary_topics: vec![topic.to_string()],
            secondary_topics: Vec::new(),
            skills: vec![
                "reading".to_string(),
                "writing-task2".to_string(),
                "speaking-part3".to_string(),
            ],
            level: None,
            teacher_pdf: None,
            private: true,
            copyright_mode: Some("private-study-only".to_string()),
            stats: None,
        },
        vocab,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_topic_article_collects_matching_entries() {
        let articles = load_articles(Path::new("../../data/articles")).unwrap();
        let topic_article = build_topic_article("health-medicine", &articles);
        assert!(!topic_article.vocab.is_empty());
        assert!(
            topic_article
                .vocab
                .iter()
                .all(|entry| entry.ielts_topics.contains(&"health-medicine".to_string()))
        );
    }
}
