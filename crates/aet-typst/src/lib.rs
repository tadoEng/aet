//! Typst PDF exporter for AET vocabulary packs.
//!
//! Strategy: generate a complete `.typ` source file with data embedded,
//! then shell out to the `typst` CLI to compile it to PDF.
//!
//! This keeps the Typst template independent of Rust and easily editable.

use aet_core::models::Article;
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

/// Check that `typst` is available in PATH.
/// Returns an error with installation instructions if not found.
pub fn check_typst_available() -> Result<()> {
    let output = Command::new("typst").arg("--version").output();
    match output {
        Ok(o) if o.status.success() => Ok(()),
        _ => bail!(
            "typst not found in PATH.\nInstall from: https://github.com/typst/typst/releases\nOr on macOS: brew install typst"
        ),
    }
}

/// Generate a vocab PDF for the given article into `out_dir`.
///
/// Steps:
/// 1. Generate `vocab-pack.typ` with embedded data.
/// 2. Shell out: `typst compile vocab-pack.typ vocab.pdf`
/// 3. Return the path to the generated PDF.
pub fn export(article: &Article, out_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(out_dir)?;

    let typ_content = generate_typ(article);
    let typ_path = out_dir.join("vocab-pack.typ");
    std::fs::write(&typ_path, typ_content.as_bytes())
        .with_context(|| format!("Cannot write {}", typ_path.display()))?;

    let pdf_path = out_dir.join("vocab.pdf");

    let status = Command::new("typst")
        .arg("compile")
        .arg(&typ_path)
        .arg(&pdf_path)
        .status()
        .context("Failed to run typst")?;

    if !status.success() {
        bail!("typst compile failed — check the generated .typ file for errors");
    }

    Ok(())
}

/// Generate the complete Typst source for a vocabulary PDF.
pub fn generate_typ(article: &Article) -> String {
    let mut doc = String::new();

    // Document setup
    doc.push_str(&format!(
        r#"#set document(title: "{title}", author: "AET")
#set page(paper: "a4", margin: (top: 20mm, bottom: 20mm, left: 20mm, right: 20mm), numbering: "1 of 1")
#set text(font: "Noto Sans", size: 9.5pt, lang: "vi")
#set table(stroke: (x, y) => if y == 0 {{ 0.5pt + rgb("#cccccc") }} else {{ 0.3pt + rgb("#e0e0e0") }})

// Header style
#show heading.where(level: 1): it => block(
  fill: rgb("#2c3e50"),
  width: 100%,
  inset: (x: 8pt, y: 6pt),
  radius: 3pt,
)[
  #text(fill: white, weight: "bold", size: 11pt)[#it.body]
]

"#,
        title = escape_typst(&article.meta.title)
    ));

    // Page header
    doc.push_str(&format!(
        r#"// Page header
#grid(
  columns: (1fr, auto),
  [#text(weight: "bold", size: 11pt)[Academic English — Thầy Hà]],
  [#text(size: 8pt, fill: rgb("#666666"))[Academic English — Thầy Hà]],
)
#v(2pt)
#text(weight: "bold", size: 13pt)[Academic Vocabulary & Collocations]
#v(2pt)
#text(size: 9pt)[Source: "#text(style: "italic")[{title}]"]
#v(1pt)
#text(size: 9pt)[{source} — {author}, {date}]
#v(12pt)

"#,
        title = escape_typst(&article.meta.title),
        source = escape_typst(&article.meta.source),
        author = escape_typst(&article.meta.author),
        date = escape_typst(&article.meta.date),
    ));

    // Category sections
    let categories = article.vocab_by_category();
    let total_categories = categories.len();

    for (cat_index, (category, entries)) in categories.iter().enumerate() {
        let cat_number = cat_index + 1;

        // Category heading
        doc.push_str(&format!(
            "= {}. {}\n\n",
            cat_number,
            escape_typst(category)
        ));

        // Table header + rows
        doc.push_str(
            r#"#table(
  columns: (20%, 30%, 30%, 20%),
  fill: (_, y) => if y == 0 { rgb("#2c3e50") } else if calc.odd(y) { rgb("#f8f9fa") } else { white },
  // Header row
  [#text(fill: white, weight: "bold")[Term]],
  [#text(fill: white, weight: "bold")[Definition]],
  [#text(fill: white, weight: "bold")[Example]],
  [#text(fill: white, weight: "bold")[Vietnamese]],
"#,
        );

        for entry in entries {
            let term = escape_typst(&entry.term);
            let definition = escape_typst(&entry.definition_en);
            let example = escape_typst(&entry.source_sentence);
            let vietnamese = escape_typst(&entry.meaning_vi);

            doc.push_str(&format!(
                "  [#text(weight: \"bold\")[{term}]],\n  [{definition}],\n  [#text(style: \"italic\")[{example}]],\n  [{vietnamese}],\n",
            ));
        }

        doc.push_str(")\n\n");

        // Add spacing between sections, not after last
        if cat_index < total_categories - 1 {
            doc.push_str("#v(8pt)\n\n");
        }
    }

    // Total count footer note
    let total_entries = article.vocab.len();
    doc.push_str(&format!(
        "\n#v(10pt)\n#align(center)[#text(size: 8pt, fill: rgb(\"#666666\"))[Total entries: {} — organised into {} thematic categories]]\n",
        total_entries, total_categories
    ));

    doc
}

/// Escape special Typst characters in a string.
///
/// Typst uses `[`, `]`, `#`, `@`, `_`, `*`, `~` as markup.
/// Inside table cells (content blocks), we need to escape these.
fn escape_typst(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('@', "\\@")
        .replace('*', "\\*")
        .replace('_', "\\_")
        .replace('~', "\\~")
        // Curly braces used in some vocab entries (e.g. "{{c1::...}}")
        .replace('{', "\\{")
        .replace('}', "\\}")
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
    fn generate_typ_contains_article_title() {
        let article = load_fixture();
        let typ = generate_typ(&article);
        assert!(typ.contains("Vietnam"));
    }

    #[test]
    fn generate_typ_contains_all_categories() {
        let article = load_fixture();
        let typ = generate_typ(&article);
        assert!(typ.contains("Demographic"));
        assert!(typ.contains("Government"));
        assert!(typ.contains("Economy"));
        assert!(typ.contains("Verb"));
        assert!(typ.contains("Adjective"));
        assert!(typ.contains("Reporting"));
        assert!(typ.contains("Useful Phrases"));
    }

    #[test]
    fn generate_typ_contains_pop001_term() {
        let article = load_fixture();
        let typ = generate_typ(&article);
        assert!(typ.contains("demographic shift"));
    }

    #[test]
    fn escape_typst_handles_brackets() {
        assert_eq!(escape_typst("a [b] c"), "a \\[b\\] c");
    }

    #[test]
    fn escape_typst_handles_hash() {
        assert_eq!(escape_typst("a #b c"), "a \\#b c");
    }
}