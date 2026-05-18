//! AET — Academic English Training CLI
//!
//! Usage:
//!   aet validate <article-path>
//!   aet build <article-path>
//!   aet build --only anki <article-path>
//!   aet build --only pdf <article-path>

mod commands;

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "aet",
    version = "0.1.0",
    about = "AET — Academic English Training content pipeline",
    long_about = "Builds vocabulary PDFs and Anki decks from structured IELTS study data."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate an article directory (article.toml + vocab.csv)
    Validate {
        /// Path to the article directory
        article_path: PathBuf,
    },
    /// Build output files from an article directory
    Build {
        /// Path to the article directory
        article_path: PathBuf,
        /// Build only a specific output type
        #[arg(long, value_name = "TYPE")]
        only: Option<BuildTarget>,
        /// Include P2/P3 rows when exporting Anki files
        #[arg(long)]
        all_priorities: bool,
    },
}

#[derive(ValueEnum, Clone, Debug)]
enum BuildTarget {
    /// Only generate Anki TSV files
    Anki,
    /// Only generate vocabulary PDF
    Pdf,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Validate { article_path } => commands::validate::run(&article_path),
        Commands::Build {
            article_path,
            only,
            all_priorities,
        } => {
            let build_anki = matches!(only, None | Some(BuildTarget::Anki));
            let build_pdf = matches!(only, None | Some(BuildTarget::Pdf));
            commands::build::run(&article_path, build_anki, build_pdf, all_priorities)
        }
    };

    if let Err(e) = result {
        eprintln!("✗ {:#}", e);
        let code = e
            .downcast_ref::<commands::CommandError>()
            .map_or(1, commands::CommandError::code);
        std::process::exit(code);
    }
}
