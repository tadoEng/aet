//! Typst PDF exporter for AET vocabulary packs.

use aet_core::{Article, IpaMode, VocabEntry};
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

pub fn check_typst_available() -> Result<()> {
    let output = Command::new("typst").arg("--version").output();
    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => bail!("typst not found in PATH. Install from https://github.com/typst/typst/releases"),
    }
}

pub fn export(article: &Article, out_dir: &Path) -> Result<()> {
    check_typst_available()?;
    std::fs::create_dir_all(out_dir)?;

    let typ_content = generate_typ(article);
    let typ_path = out_dir.join("vocab-pack.typ");
    std::fs::write(&typ_path, typ_content.as_bytes())
        .with_context(|| format!("Cannot write {}", typ_path.display()))?;

    let pdf_path = out_dir.join("vocabulary.pdf");
    let status = Command::new("typst")
        .arg("compile")
        .arg(&typ_path)
        .arg(&pdf_path)
        .status()
        .context("Failed to run typst")?;

    if !status.success() {
        bail!("typst compile failed; check {}", typ_path.display());
    }

    Ok(())
}

pub fn generate_typ(article: &Article) -> String {
    let ipa_mode = article.ipa_mode();
    let mut doc = String::new();

    doc.push_str(&format!(
        r##"#set document(title: "{title}", author: "AET")
#set page(paper: "a4", margin: (top: 16mm, bottom: 18mm, left: 14mm, right: 14mm), numbering: "1")
#set text(size: 9pt, lang: "vi")
#set heading(numbering: none)

#align(center)[
  #text(size: 16pt, weight: "bold")[Academic Vocabulary & Collocations] \
  #text(size: 10pt)[{article_title}] \
  #text(size: 8pt, fill: rgb("#666666"))[{source_line}]
]

#v(8pt)
#text(size: 8pt, fill: rgb("#666666"))[IPA mode: {ipa_mode} | Total entries: {entry_count}]
#v(8pt)

"##,
        title = escape_typst_string(&article.meta.title),
        article_title = escape_typst_content(&article.meta.title),
        source_line = escape_typst_content(&source_line(article)),
        ipa_mode = ipa_mode,
        entry_count = article.vocab.len(),
    ));

    for (index, (category, entries)) in article.vocab_by_category().iter().enumerate() {
        doc.push_str(&format!(
            "== {}. {}\n\n",
            index + 1,
            escape_typst_content(category)
        ));
        append_table(&mut doc, ipa_mode, entries);
        doc.push_str("\n#v(8pt)\n\n");
    }

    doc
}

fn append_table(doc: &mut String, ipa_mode: IpaMode, entries: &[&VocabEntry]) {
    if ipa_mode.has_column() {
        doc.push_str(
            r##"#table(
  columns: (1.15fr, 0.85fr, 1.45fr, 1.65fr, 1fr),
  inset: 4pt,
  stroke: (_, y) => if y == 0 { 0.7pt + rgb("#b7c2cc") } else { 0.35pt + rgb("#dde3ea") },
  fill: (_, y) => if y == 0 { rgb("#263645") } else if calc.odd(y) { rgb("#f5f7f9") } else { white },
  [#text(fill: white, weight: "bold")[Term]],
  [#text(fill: white, weight: "bold")[IPA]],
  [#text(fill: white, weight: "bold")[Definition]],
  [#text(fill: white, weight: "bold")[Example]],
  [#text(fill: white, weight: "bold")[Vietnamese]],
"##,
        );
        for entry in entries {
            doc.push_str(&format!(
                "  [#text(weight: \"bold\")[{}]],\n  [{}],\n  [{}],\n  [#text(style: \"italic\")[{}]],\n  [{}],\n",
                escape_typst_content(&entry.term),
                escape_typst_content(entry.ipa.as_deref().unwrap_or("-")),
                escape_typst_content(&entry.definition_en),
                escape_typst_content(&entry.source_sentence),
                escape_typst_content(&entry.meaning_vi),
            ));
        }
    } else {
        doc.push_str(
            r##"#table(
  columns: (1.15fr, 1.45fr, 1.65fr, 1fr),
  inset: 4pt,
  stroke: (_, y) => if y == 0 { 0.7pt + rgb("#b7c2cc") } else { 0.35pt + rgb("#dde3ea") },
  fill: (_, y) => if y == 0 { rgb("#263645") } else if calc.odd(y) { rgb("#f5f7f9") } else { white },
  [#text(fill: white, weight: "bold")[Term]],
  [#text(fill: white, weight: "bold")[Definition]],
  [#text(fill: white, weight: "bold")[Example]],
  [#text(fill: white, weight: "bold")[Vietnamese]],
"##,
        );
        for entry in entries {
            doc.push_str(&format!(
                "  [#text(weight: \"bold\")[{}]],\n  [{}],\n  [#text(style: \"italic\")[{}]],\n  [{}],\n",
                escape_typst_content(&entry.term),
                escape_typst_content(&entry.definition_en),
                escape_typst_content(&entry.source_sentence),
                escape_typst_content(&entry.meaning_vi),
            ));
        }
    }
    doc.push_str(")\n");
}

fn source_line(article: &Article) -> String {
    let author = article.meta.author.as_deref().unwrap_or("Unknown author");
    format!(
        "{} - {}, {}",
        article.meta.source_name, author, article.meta.date
    )
}

fn escape_typst_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn escape_typst_content(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('#', "\\#")
        .replace('@', "\\@")
        .replace('*', "\\*")
        .replace('$', "\\$")
        .replace('_', "\\_")
        .replace('~', "\\~")
        .replace('{', "\\{")
        .replace('}', "\\}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use aet_core::{parse_article_meta, parse_vocab};

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
    fn generate_typ_contains_article_title() {
        let article = load_fixture();
        let typ = generate_typ(&article);
        assert!(typ.contains("Facing a Demographic Shift"));
    }

    #[test]
    fn generate_typ_omits_ipa_column_when_absent() {
        let article = load_fixture();
        let typ = generate_typ(&article);
        assert!(!typ.contains("[#text(fill: white, weight: \"bold\")[IPA]]"));
    }

    #[test]
    fn generate_typ_includes_ipa_column_when_mixed() {
        let mut article = load_fixture();
        article.vocab[0].ipa = Some("/test/".to_string());
        let typ = generate_typ(&article);
        assert!(typ.contains("[#text(fill: white, weight: \"bold\")[IPA]]"));
        assert!(typ.contains("[-]"));
    }

    #[test]
    fn escape_typst_content_handles_markup_characters() {
        let escaped = escape_typst_content("a [b] #tag $120");
        assert_eq!(escaped, "a \\[b\\] \\#tag \\$120");
    }
}
