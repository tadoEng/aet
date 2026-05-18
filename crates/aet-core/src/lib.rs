//! AET Core — data models, parsing, and validation.
//!
//! This crate is the foundation. All other AET crates depend on it.
//! It has no knowledge of output formats (Anki, Typst, JSON).

pub mod models;
pub mod validator;

pub use models::{
    Article, ArticleMeta, CardType, EntryType, IpaMode, Origin, Priority, ReviewStatus, VocabEntry,
    parse_article_meta, parse_vocab,
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
