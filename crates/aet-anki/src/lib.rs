//! Anki TSV exporters for AET vocabulary entries.

use aet_core::{Article, CardType, VocabEntry};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct AnkiExportResult {
    pub basic_count: usize,
    pub cloze_count: usize,
    pub production_count: usize,
    pub cloze_fallback_count: usize,
}

pub fn generate_basic_tsv(article: &Article, all_priorities: bool) -> Result<(String, usize)> {
    let rows = exportable_entries(article, all_priorities)
        .filter(|entry| entry.card_types.contains(&CardType::Basic))
        .map(|entry| {
            tsv_row([
                format!("What does \"{}\" mean?", entry.term),
                build_meaning_back(entry),
                build_reference_extra(entry),
                build_tags(entry, &article.meta.id),
            ])
        })
        .collect::<Vec<_>>();

    let count = rows.len();
    Ok((rows.join("\n"), count))
}

pub fn generate_cloze_tsv(
    article: &Article,
    all_priorities: bool,
) -> Result<(String, usize, usize)> {
    let mut fallback_count = 0;
    let rows = exportable_entries(article, all_priorities)
        .filter(|entry| entry.card_types.contains(&CardType::Cloze))
        .map(|entry| {
            let (front, used_fallback) = build_cloze_front(entry);
            if used_fallback {
                fallback_count += 1;
            }
            tsv_row([
                front,
                entry.term.clone(),
                build_cloze_extra(entry),
                build_tags(entry, &article.meta.id),
            ])
        })
        .collect::<Vec<_>>();

    let count = rows.len();
    Ok((rows.join("\n"), count, fallback_count))
}

pub fn generate_production_tsv(article: &Article, all_priorities: bool) -> Result<(String, usize)> {
    let rows = exportable_entries(article, all_priorities)
        .filter(|entry| entry.card_types.contains(&CardType::Production))
        .map(|entry| {
            let topic = entry
                .ielts_topics
                .first()
                .map(String::as_str)
                .unwrap_or("academic English");
            tsv_row([
                format!(
                    "Use \"{}\" in an IELTS-style sentence about {}.",
                    entry.term, topic
                ),
                build_meaning_back(entry),
                build_production_extra(entry),
                build_tags(entry, &article.meta.id),
            ])
        })
        .collect::<Vec<_>>();

    let count = rows.len();
    Ok((rows.join("\n"), count))
}

pub fn export(article: &Article, out_dir: &Path, all_priorities: bool) -> Result<AnkiExportResult> {
    std::fs::create_dir_all(out_dir)?;

    let (basic_tsv, basic_count) = generate_basic_tsv(article, all_priorities)?;
    std::fs::write(out_dir.join("anki-basic.tsv"), basic_tsv.as_bytes())?;

    let (cloze_tsv, cloze_count, cloze_fallback_count) =
        generate_cloze_tsv(article, all_priorities)?;
    std::fs::write(out_dir.join("anki-cloze.tsv"), cloze_tsv.as_bytes())?;

    let (production_tsv, production_count) = generate_production_tsv(article, all_priorities)?;
    std::fs::write(
        out_dir.join("anki-production.tsv"),
        production_tsv.as_bytes(),
    )?;
    std::fs::write(
        out_dir.join("anki-import-guide.md"),
        build_import_guide(&article.meta.id).as_bytes(),
    )?;
    write_template_files(out_dir)?;

    Ok(AnkiExportResult {
        basic_count,
        cloze_count,
        production_count,
        cloze_fallback_count,
    })
}

fn write_template_files(out_dir: &Path) -> Result<()> {
    let template_dir = out_dir.join("anki-templates");
    std::fs::create_dir_all(&template_dir)?;
    std::fs::write(template_dir.join("basic-front.html"), BASIC_FRONT_TEMPLATE)?;
    std::fs::write(template_dir.join("basic-back.html"), BASIC_BACK_TEMPLATE)?;
    std::fs::write(
        template_dir.join("production-front.html"),
        PRODUCTION_FRONT_TEMPLATE,
    )?;
    std::fs::write(
        template_dir.join("production-back.html"),
        PRODUCTION_BACK_TEMPLATE,
    )?;
    std::fs::write(template_dir.join("cloze-front.html"), CLOZE_FRONT_TEMPLATE)?;
    std::fs::write(template_dir.join("cloze-back.html"), CLOZE_BACK_TEMPLATE)?;
    std::fs::write(template_dir.join("card-styling.css"), CARD_STYLING)?;
    Ok(())
}

fn build_import_guide(article_id: &str) -> String {
    format!(
        r#"# AET Anki Import Guide

Article: `{article_id}`

The TSV files contain note data only. The learning experience is controlled by the Anki note type templates in `anki-templates/`.

## Recommended Import Setup

Create these custom note types in Anki before importing:

1. `AET Basic Type Answer`
2. `AET Production Type Answer`
3. `AET Cloze Type Answer`

`AET Basic Type Answer` and `AET Production Type Answer` need these fields, in this order:

```text
Front
Back
Extra
Tags
```

When importing `anki-basic.tsv` or `anki-production.tsv`:

- Type: `AET Basic Type Answer` or `AET Production Type Answer`
- Field separator: Tab
- Field 1 -> `Front`
- Field 2 -> `Back`
- Field 3 -> `Extra`
- Field 4 -> `Tags`
- Tags column: field 4

Import `anki-cloze.tsv` with `AET Cloze Type Answer` if you want a typed-answer box, or Anki's built-in `Cloze` note type if you only want normal reveal cards:

`AET Cloze Type Answer` needs these fields, in this order:

```text
Text
Back Extra
Extra
Tags
```

- Field 1 -> `Text`
- Field 2 -> `Back Extra`
- Field 3 -> `Extra`
- Field 4 -> `Tags`

## AET Basic Type Answer

Use:

- Front: `anki-templates/basic-front.html`
- Back: `anki-templates/basic-back.html`
- Styling: `anki-templates/card-styling.css`

## AET Production Type Answer

Use:

- Front: `anki-templates/production-front.html`
- Back: `anki-templates/production-back.html`
- Styling: `anki-templates/card-styling.css`

## AET Cloze Type Answer

Clone Anki's built-in `Cloze` note type, then use:

- Front: `anki-templates/cloze-front.html`
- Back: `anki-templates/cloze-back.html`
- Styling: `anki-templates/card-styling.css`

## Learning Behavior

- Basic and production cards use an open response box. Your attempt is saved locally and shown on the back for self-comparison.
- Cloze cards use Anki's native typed-answer comparison with `{{{{type:cloze:Text}}}}`.
- Hints are hidden in disclosure panels so they do not steal the recall attempt.
- Final grading stays manual through Anki's Again/Hard/Good/Easy buttons.

## Multiple Choice

Anki does not have native multiple-choice grading for TSV imports. You can make a visual multiple-choice card with custom HTML/JavaScript, but it will not behave like Anki's typed-answer comparison. For AET v0.1, typed answer is the reliable option.
"#
    )
}

const BASIC_FRONT_TEMPLATE: &str = r#"<div class="aet-card" data-aet-mode="basic">
  <div class="aet-kicker">AET Basic Recall</div>
  <div class="aet-prompt">{{Front}}</div>
  <div class="aet-instruction">Recall the meaning in English or Vietnamese before revealing the answer.</div>

  <label class="aet-label" for="aet-open-answer">Your recall attempt</label>
  <textarea id="aet-open-answer" class="aet-open-answer" rows="4" placeholder="Write the meaning from memory. A short paraphrase is enough."></textarea>

  <div class="aet-footer-note">Then press Show Answer and compare your attempt with the reference.</div>
</div>

<script>
(function () {
  var key = "aet-basic-" + document.querySelector(".aet-prompt").textContent.trim();
  var box = document.getElementById("aet-open-answer");
  box.value = sessionStorage.getItem(key) || "";
  box.addEventListener("input", function () {
    sessionStorage.setItem(key, box.value);
  });
})();
</script>"#;

const BASIC_BACK_TEMPLATE: &str = r#"<div class="aet-card" data-aet-mode="basic">
  <div class="aet-kicker">AET Basic Recall</div>
  <div class="aet-prompt">{{Front}}</div>

  <section class="aet-panel aet-attempt-panel">
    <div class="aet-panel-title">Your attempt</div>
    <div id="aet-saved-attempt" class="aet-attempt-text">No typed attempt captured.</div>
  </section>

  <section class="aet-panel aet-answer-panel">
    <div class="aet-panel-title">Reference answer</div>
    <div class="aet-answer">{{Back}}</div>
  </section>

  <details class="aet-details" open>
    <summary>Source, pattern, and topics</summary>
    <div class="aet-extra">{{Extra}}</div>
  </details>
</div>

<script>
(function () {
  var key = "aet-basic-" + document.querySelector(".aet-prompt").textContent.trim();
  var attempt = sessionStorage.getItem(key);
  if (attempt && attempt.trim()) {
    document.getElementById("aet-saved-attempt").textContent = attempt;
  }
})();
</script>"#;

const PRODUCTION_FRONT_TEMPLATE: &str = r#"<div class="aet-card" data-aet-mode="production">
  <div class="aet-kicker">AET Production Sprint</div>
  <div class="aet-prompt">{{Front}}</div>
  <div class="aet-instruction">Write one sentence you could realistically use in IELTS Writing or Speaking.</div>

  <label class="aet-label" for="aet-open-answer">Your sentence</label>
  <textarea id="aet-open-answer" class="aet-open-answer" rows="5" placeholder="Try a complete sentence before checking the reference."></textarea>

  <details class="aet-details aet-subtle-details">
    <summary>Need a nudge?</summary>
    <div class="aet-extra">{{Extra}}</div>
  </details>
</div>

<script>
(function () {
  var key = "aet-production-" + document.querySelector(".aet-prompt").textContent.trim();
  var box = document.getElementById("aet-open-answer");
  box.value = sessionStorage.getItem(key) || "";
  box.addEventListener("input", function () {
    sessionStorage.setItem(key, box.value);
  });
})();
</script>"#;

const PRODUCTION_BACK_TEMPLATE: &str = r#"<div class="aet-card" data-aet-mode="production">
  <div class="aet-kicker">AET Production Sprint</div>
  <div class="aet-prompt">{{Front}}</div>

  <section class="aet-panel aet-attempt-panel">
    <div class="aet-panel-title">Your sentence</div>
    <div id="aet-saved-attempt" class="aet-attempt-text">No typed attempt captured.</div>
  </section>

  <section class="aet-panel aet-answer-panel">
    <div class="aet-panel-title">Meaning anchor</div>
    <div class="aet-answer">{{Back}}</div>
  </section>

  <details class="aet-details" open>
    <summary>Reference panel</summary>
    <div class="aet-extra">{{Extra}}</div>
  </details>
</div>

<script>
(function () {
  var key = "aet-production-" + document.querySelector(".aet-prompt").textContent.trim();
  var attempt = sessionStorage.getItem(key);
  if (attempt && attempt.trim()) {
    document.getElementById("aet-saved-attempt").textContent = attempt;
  }
})();
</script>"#;

const CLOZE_FRONT_TEMPLATE: &str = r#"<div class="aet-card" data-aet-mode="cloze">
  <div class="aet-kicker">AET Cloze Recall</div>
  <div class="aet-prompt">{{cloze:Text}}</div>
  <div class="aet-instruction">Type the missing word or phrase exactly enough for Anki to compare it.</div>
  <div class="aet-typebox">{{type:cloze:Text}}</div>
</div>"#;

const CLOZE_BACK_TEMPLATE: &str = r#"<div class="aet-card" data-aet-mode="cloze">
  <div class="aet-kicker">AET Cloze Recall</div>
  <div class="aet-prompt">{{cloze:Text}}</div>
  <div class="aet-typebox">{{type:cloze:Text}}</div>

  <section class="aet-panel aet-answer-panel">
    <div class="aet-panel-title">Answer and meaning</div>
    <div class="aet-answer">{{Back Extra}}</div>
  </section>

  <details class="aet-details" open>
    <summary>Reference panel</summary>
    <div class="aet-extra">{{Extra}}</div>
  </details>
</div>"#;

const CARD_STYLING: &str = r#".card {
  margin: 0;
  padding: 0;
  font-family: Aptos, "Segoe UI", "Noto Sans", sans-serif;
  font-size: 20px;
  text-align: center;
  color: #f4f1e8;
  background: #252525;
}

.aet-card {
  max-width: 920px;
  margin: 28px auto;
  padding: 28px;
  line-height: 1.5;
}

.aet-kicker {
  display: inline-block;
  margin-bottom: 18px;
  padding: 4px 10px;
  border: 1px solid #8f866f;
  border-radius: 999px;
  color: #d9c991;
  font-size: 12px;
  text-transform: uppercase;
}

.aet-prompt {
  margin: 0 auto 18px;
  max-width: 820px;
  font-size: 28px;
  font-weight: 650;
}

.aet-instruction,
.aet-footer-note {
  margin: 12px auto;
  max-width: 720px;
  color: #c8c2b4;
  font-size: 16px;
}

.aet-label {
  display: block;
  margin: 24px 0 8px;
  color: #d9c991;
  font-size: 14px;
  text-transform: uppercase;
}

.aet-open-answer,
input[type="text"] {
  box-sizing: border-box;
  width: 100%;
  max-width: 820px;
  padding: 16px 18px;
  border: 1px solid #7f7869;
  border-radius: 8px;
  color: #f4f1e8;
  background: #1d1d1d;
  font-size: 22px;
  text-align: left;
  outline: none;
}

input[type="text"] {
  text-align: center;
}

.aet-open-answer:focus,
input[type="text"]:focus {
  border-color: #d9c991;
  box-shadow: 0 0 0 3px rgba(217, 201, 145, 0.18);
}

.aet-panel,
.aet-details {
  max-width: 820px;
  margin: 18px auto;
  padding: 16px 18px;
  border: 1px solid #625d52;
  border-radius: 8px;
  background: #2d2d2b;
  text-align: left;
}

.aet-panel-title {
  margin-bottom: 8px;
  color: #d9c991;
  font-size: 13px;
  font-weight: 700;
  text-transform: uppercase;
}

.aet-attempt-text {
  white-space: pre-wrap;
  color: #f4f1e8;
}

.aet-answer {
  color: #a7e08f;
  font-size: 22px;
}

.aet-extra {
  color: #ded7c8;
  font-size: 16px;
}

.aet-details summary {
  cursor: pointer;
  color: #d9c991;
  font-weight: 700;
}

.aet-subtle-details {
  opacity: 0.86;
}

.typeGood {
  color: #a7e08f;
}

.typeBad {
  color: #ff9b8f;
}

.typeMissed {
  color: #d9c991;
}

code {
  color: #d9c991;
}"#;

fn exportable_entries(
    article: &Article,
    all_priorities: bool,
) -> impl Iterator<Item = &VocabEntry> {
    article
        .vocab
        .iter()
        .filter(move |entry| entry.should_export_to_anki(all_priorities))
}

fn build_meaning_back(entry: &VocabEntry) -> String {
    format!("{}<br>{}", entry.definition_en, entry.meaning_vi)
}

fn build_reference_extra(entry: &VocabEntry) -> String {
    let mut lines = vec![format!("Source: {}", entry.source_sentence)];
    if let Some(pattern) = &entry.collocation_pattern {
        lines.push(format!("Pattern: {}", pattern));
    }
    lines.push(format!("Topic: {}", entry.ielts_topics.join(" · ")));
    lines.join("<br>")
}

fn build_cloze_extra(entry: &VocabEntry) -> String {
    let mut lines = vec![format!("{} = {}", entry.term, entry.definition_en)];
    lines.push(format!("Vietnamese: {}", entry.meaning_vi));
    if let Some(pattern) = &entry.collocation_pattern {
        lines.push(format!("Pattern: {}", pattern));
    }
    lines.push(format!("Topic: {}", entry.ielts_topics.join(" · ")));
    lines.join("<br>")
}

fn build_production_extra(entry: &VocabEntry) -> String {
    let mut lines = Vec::new();
    if let Some(pattern) = &entry.collocation_pattern {
        lines.push(format!("Pattern: {}", pattern));
    }
    lines.push(format!("Source: {}", entry.source_sentence));
    lines.push(format!("Topic: {}", entry.ielts_topics.join(" · ")));
    if let Some(my_sentence) = &entry.my_sentence {
        lines.push(format!("Your previous sentence:<br>{}", my_sentence));
    }
    lines.join("<br>")
}

fn build_cloze_front(entry: &VocabEntry) -> (String, bool) {
    if let Some(cloze) = replace_term_with_cloze(&entry.source_sentence, &entry.term) {
        return (cloze, false);
    }

    if let Some(cloze) = replace_term_case_insensitive(&entry.source_sentence, &entry.term) {
        return (cloze, false);
    }

    if let Some(first_word) = entry.term.split_whitespace().next() {
        if let Some(cloze) = replace_term_case_insensitive(&entry.source_sentence, first_word) {
            return (cloze, false);
        }
    }

    (format!("{{{{c1::{}}}}}", entry.source_sentence), true)
}

fn replace_term_with_cloze(sentence: &str, term: &str) -> Option<String> {
    sentence
        .find(term)
        .map(|start| replace_range(sentence, start, term.len()))
}

fn replace_term_case_insensitive(sentence: &str, term: &str) -> Option<String> {
    let lower_sentence = sentence.to_lowercase();
    let lower_term = term.to_lowercase();
    lower_sentence
        .find(&lower_term)
        .map(|start| replace_range(sentence, start, term.len()))
}

fn replace_range(sentence: &str, start: usize, len: usize) -> String {
    let end = start + len;
    let original = &sentence[start..end];
    format!(
        "{}{{{{c1::{}}}}}{}",
        &sentence[..start],
        original,
        &sentence[end..]
    )
}

fn build_tags(entry: &VocabEntry, article_id: &str) -> String {
    let topic = entry
        .ielts_topics
        .first()
        .map(String::as_str)
        .unwrap_or("unknown-topic");
    let mut tags = vec![
        format!("AET"),
        format!("article::{}", article_id),
        format!("topic::{}", topic),
        format!("priority::{}", entry.priority),
    ];
    for tag in &entry.tags {
        let normalized = tag.replace(' ', "-");
        if !tags.contains(&normalized) {
            tags.push(normalized);
        }
    }
    tags.join(" ")
}

fn tsv_row(fields: [String; 4]) -> String {
    fields.map(sanitize_tsv_field).join("\t")
}

fn sanitize_tsv_field(field: String) -> String {
    field
        .replace('\t', " ")
        .replace("\r\n", "<br>")
        .replace('\n', "<br>")
        .replace('\r', "<br>")
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
    fn generate_basic_tsv_should_produce_four_column_rows() {
        let article = load_fixture();
        let (tsv, count) = generate_basic_tsv(&article, false).unwrap();
        assert!(count > 0);
        assert_eq!(tsv.lines().next().unwrap().split('\t').count(), 4);
    }

    #[test]
    fn generate_cloze_tsv_should_produce_cloze_rows() {
        let article = load_fixture();
        let (tsv, count, _fallbacks) = generate_cloze_tsv(&article, false).unwrap();
        assert!(count > 0);
        assert!(tsv.contains("{{c1::"));
    }

    #[test]
    fn generate_production_tsv_should_use_prompt_card_format() {
        let article = load_fixture();
        let (tsv, count) = generate_production_tsv(&article, false).unwrap();
        assert!(count > 0);
        assert!(tsv.lines().next().unwrap().contains("Use \""));
    }

    #[test]
    fn default_export_filter_only_exports_p1_approved_yes_rows() {
        let article = load_fixture();
        let expected = article
            .vocab
            .iter()
            .filter(|entry| entry.should_export_to_anki(false))
            .filter(|entry| entry.card_types.contains(&CardType::Basic))
            .count();
        let (_tsv, count) = generate_basic_tsv(&article, false).unwrap();
        assert_eq!(count, expected);
    }

    #[test]
    fn all_priorities_exports_more_or_equal_rows() {
        let article = load_fixture();
        let (_default_tsv, default_count) = generate_basic_tsv(&article, false).unwrap();
        let (_all_tsv, all_count) = generate_basic_tsv(&article, true).unwrap();
        assert!(all_count >= default_count);
    }

    #[test]
    fn import_guide_includes_type_answer_template() {
        let guide = build_import_guide("article-id");
        assert!(guide.contains("anki-templates/basic-front.html"));
        assert!(BASIC_FRONT_TEMPLATE.contains("textarea"));
        assert!(CLOZE_FRONT_TEMPLATE.contains("{{type:cloze:Text}}"));
    }

    #[test]
    fn replace_term_with_cloze_finds_exact_term() {
        let result = replace_term_with_cloze(
            "Vietnam is facing a demographic shift as its population ages.",
            "demographic shift",
        );
        assert!(result.unwrap().contains("{{c1::demographic shift}}"));
    }

    #[test]
    fn cloze_fallback_wraps_whole_sentence() {
        let article = load_fixture();
        let mut entry = article.vocab[0].clone();
        entry.term = "not in the example".to_string();
        let (front, used_fallback) = build_cloze_front(&entry);
        assert!(used_fallback);
        assert!(front.starts_with("{{c1::"));
    }
}
