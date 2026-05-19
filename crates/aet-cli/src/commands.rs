use aet_core::{
    IELTS_TOPICS, build_topic_article, load_article, load_articles, validate as validate_article,
};
use anyhow::Result;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct CommandError {
    code: i32,
    message: String,
}

impl CommandError {
    pub fn code(&self) -> i32 {
        self.code
    }

    fn validation(message: impl Into<String>) -> Self {
        Self {
            code: 1,
            message: message.into(),
        }
    }

    fn build(message: impl Into<String>) -> Self {
        Self {
            code: 2,
            message: message.into(),
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CommandError {}

pub mod validate {
    use super::*;

    pub fn run(article_path: &Path) -> Result<()> {
        let timer = SessionTimer::start("validate", article_path);
        let result = run_inner(article_path);
        timer.finish();
        result
    }

    fn run_inner(article_path: &Path) -> Result<()> {
        let article = load_article(article_path).map_err(|error| {
            CommandError::validation(format!("Could not load article: {:#}", error))
        })?;
        let validation = validate_article(&article);
        validation.print_summary();
        if validation.is_valid() {
            Ok(())
        } else {
            Err(CommandError::validation(format!(
                "{} validation errors found for {}",
                validation.errors.len(),
                validation.article_id
            ))
            .into())
        }
    }
}

pub mod build {
    use super::*;

    pub fn run(
        article_path: &Path,
        build_anki: bool,
        build_pdf: bool,
        all_priorities: bool,
    ) -> Result<()> {
        let timer = SessionTimer::start("build", article_path);
        let result = run_inner(article_path, build_anki, build_pdf, all_priorities);
        timer.finish();
        result
    }

    fn run_inner(
        article_path: &Path,
        build_anki: bool,
        build_pdf: bool,
        all_priorities: bool,
    ) -> Result<()> {
        let article = load_article(article_path).map_err(|error| {
            CommandError::validation(format!("Could not load article: {:#}", error))
        })?;
        let validation = validate_article(&article);
        validation.print_summary();
        if !validation.is_valid() {
            return Err(CommandError::validation(format!(
                "{} validation errors found for {}",
                validation.errors.len(),
                validation.article_id
            ))
            .into());
        }

        let out_dir = PathBuf::from("dist").join(&article.meta.id);
        println!("[aet] Building {}...", article.meta.id);

        if build_anki {
            let result = aet_anki::export(&article, &out_dir, all_priorities)
                .map_err(|error| CommandError::build(format!("Anki export failed: {:#}", error)))?;
            println!("[aet] Output: {}", out_dir.join("anki-basic.tsv").display());
            println!("[aet] Output: {}", out_dir.join("anki-cloze.tsv").display());
            println!(
                "[aet] Output: {}",
                out_dir.join("anki-production.tsv").display()
            );
            println!(
                "{}",
                format_anki_export_summary(
                    result.basic_count,
                    result.cloze_count,
                    result.production_count,
                    result.cloze_fallback_count
                )
            );
        }

        if build_pdf {
            aet_typst::export(&article, &out_dir)
                .map_err(|error| CommandError::build(format!("PDF export failed: {:#}", error)))?;
            println!("[aet] Output: {}", out_dir.join("vocabulary.pdf").display());
        }

        Ok(())
    }
}

fn format_anki_export_summary(
    basic_count: usize,
    cloze_count: usize,
    production_count: usize,
    cloze_fallback_count: usize,
) -> String {
    let mut summary = format!(
        "[aet] Anki rows: basic={}, cloze={}, production={} (cloze fallbacks={})",
        basic_count, cloze_count, production_count, cloze_fallback_count
    );
    if cloze_fallback_count > 0 {
        summary.push_str(&format!(
            "\n[aet] ⚠ {} cloze fallback card(s) wrapped a whole sentence; review manually before import.",
            cloze_fallback_count
        ));
    }
    summary
}

pub mod build_topic {
    use super::*;

    pub fn run(topic: &str, build_anki: bool, build_pdf: bool, all_priorities: bool) -> Result<()> {
        let timer = SessionTimer::start("build-topic", Path::new(topic));
        let result = run_inner(topic, build_anki, build_pdf, all_priorities);
        timer.finish();
        result
    }

    fn run_inner(
        topic: &str,
        build_anki: bool,
        build_pdf: bool,
        all_priorities: bool,
    ) -> Result<()> {
        if !IELTS_TOPICS.contains(&topic) {
            return Err(CommandError::validation(format!(
                "Unknown topic '{}'. Expected one of: {}",
                topic,
                IELTS_TOPICS.join(", ")
            ))
            .into());
        }

        let articles = load_articles(Path::new("data/articles")).map_err(|error| {
            CommandError::validation(format!("Could not load data/articles: {:#}", error))
        })?;
        let topic_article = build_topic_article(topic, &articles);
        let contributing_articles = articles
            .iter()
            .filter(|article| {
                article.vocab.iter().any(|entry| {
                    entry
                        .ielts_topics
                        .iter()
                        .any(|entry_topic| entry_topic == topic)
                })
            })
            .count();
        if topic_article.vocab.is_empty() {
            return Err(CommandError::validation(format!(
                "No vocabulary rows found for topic '{}'",
                topic
            ))
            .into());
        }

        let validation = validate_article(&topic_article);
        validation.print_summary();
        if !validation.is_valid() {
            return Err(CommandError::validation(format!(
                "{} validation errors found for topic {}",
                validation.errors.len(),
                topic
            ))
            .into());
        }

        let out_dir = PathBuf::from("dist").join("topics").join(topic);
        println!(
            "[aet] Building topic bank {} from {} row(s) across {} contributing article(s) (scanned {})...",
            topic,
            topic_article.vocab.len(),
            contributing_articles,
            articles.len()
        );

        if build_anki {
            let result = aet_anki::export(&topic_article, &out_dir, all_priorities)
                .map_err(|error| CommandError::build(format!("Anki export failed: {:#}", error)))?;
            println!("[aet] Output: {}", out_dir.join("anki-basic.tsv").display());
            println!("[aet] Output: {}", out_dir.join("anki-cloze.tsv").display());
            println!(
                "[aet] Output: {}",
                out_dir.join("anki-production.tsv").display()
            );
            println!(
                "{}",
                format_anki_export_summary(
                    result.basic_count,
                    result.cloze_count,
                    result.production_count,
                    result.cloze_fallback_count
                )
            );
        }

        if build_pdf {
            aet_typst::export(&topic_article, &out_dir)
                .map_err(|error| CommandError::build(format!("PDF export failed: {:#}", error)))?;
            println!("[aet] Output: {}", out_dir.join("vocabulary.pdf").display());
        }

        Ok(())
    }
}

struct SessionTimer {
    path: PathBuf,
    command: String,
    article: String,
    start: Instant,
}

impl SessionTimer {
    fn start(command: &str, article_path: &Path) -> Self {
        let date = current_utc_date();
        let path = std::env::temp_dir().join(format!("aet-session-{}.json", date));
        let article = article_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string();
        Self {
            path,
            command: command.to_string(),
            article,
            start: Instant::now(),
        }
    }

    #[cfg(test)]
    fn with_path(path: PathBuf, command: &str, article: &str) -> Self {
        Self {
            path,
            command: command.to_string(),
            article: article.to_string(),
            start: Instant::now(),
        }
    }

    fn finish(&self) {
        let elapsed_ms = self.start.elapsed().as_millis() as u64;
        let previous_total = read_total_elapsed_ms(&self.path).unwrap_or(0);
        let total_elapsed_ms = previous_total.saturating_add(elapsed_ms);
        let previous_commands = read_command_history(&self.path).unwrap_or_default();
        let current_command = format!(
            "    {{ \"cmd\": \"{}\", \"article\": \"{}\", \"elapsed_ms\": {} }}",
            json_escape(&self.command),
            json_escape(&self.article),
            elapsed_ms
        );
        let commands = if previous_commands.trim().is_empty() {
            current_command
        } else {
            format!("{},\n{}", previous_commands.trim_end(), current_command)
        };

        let payload = format!(
            "{{\n  \"date\": \"{}\",\n  \"commands\": [\n{}\n  ],\n  \"total_elapsed_ms\": {}\n}}\n",
            current_utc_date(),
            commands,
            total_elapsed_ms
        );

        if let Err(error) = std::fs::write(&self.path, payload) {
            eprintln!(
                "⚠ Could not write session timer {}: {}",
                self.path.display(),
                error
            );
            return;
        }

        println!("[aet] Done in {:.1}s.", elapsed_ms as f64 / 1000.0);
        let total_minutes = total_elapsed_ms / 60_000;
        if total_minutes > 20 {
            println!(
                "[aet] ⚠ Session time today: {} min. Stop here - continue tomorrow.",
                total_minutes
            );
        } else {
            println!(
                "[aet] Session time today: {} min. ✓ Within 20-min budget.",
                total_minutes
            );
        }
    }
}

fn read_command_history(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let marker = "\"commands\"";
    let marker_start = content.find(marker)?;
    let array_start = content[marker_start..].find('[')? + marker_start + 1;
    let array_end = content[array_start..].find("\n  ]")? + array_start;
    let commands = content[array_start..array_end].trim();
    if commands.is_empty() {
        Some(String::new())
    } else {
        Some(commands.to_string())
    }
}

fn read_total_elapsed_ms(path: &Path) -> Option<u64> {
    let content = std::fs::read_to_string(path).ok()?;
    let marker = "\"total_elapsed_ms\":";
    let start = content.find(marker)? + marker.len();
    let rest = content[start..].trim_start();
    let digits = rest
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn current_utc_date() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 86_400)
        .unwrap_or(0);
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}")
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    let year = y + i64::from(m <= 2);
    (year as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_matches_unix_epoch() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn read_total_elapsed_ms_reads_session_file() {
        let path = std::env::temp_dir().join("aet-session-test-read.json");
        std::fs::write(&path, "{ \"total_elapsed_ms\": 1234 }").unwrap();
        assert_eq!(read_total_elapsed_ms(&path), Some(1234));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn session_timer_accumulates_existing_total() {
        let path = std::env::temp_dir().join("aet-session-test-accumulate.json");
        std::fs::write(&path, "{ \"total_elapsed_ms\": 1000 }").unwrap();
        let timer = SessionTimer::with_path(path.clone(), "validate", "article");
        timer.finish();
        let total = read_total_elapsed_ms(&path).unwrap();
        assert!(total >= 1000);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn session_timer_preserves_command_history() {
        let path = std::env::temp_dir().join("aet-session-test-history.json");
        let _ = std::fs::remove_file(&path);

        SessionTimer::with_path(path.clone(), "validate", "vietnam").finish();
        SessionTimer::with_path(path.clone(), "build", "vietnam").finish();

        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("\"cmd\": \"validate\""));
        assert!(content.contains("\"cmd\": \"build\""));
        assert_eq!(content.matches("\"cmd\"").count(), 2);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn anki_export_summary_warns_when_cloze_fallbacks_exist() {
        assert!(format_anki_export_summary(3, 3, 3, 2).contains("review manually"));
    }
}
