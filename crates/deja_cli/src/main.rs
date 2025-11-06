//! Deja-vu CLI - Code duplication detector
//!
//! An extremely fast code duplication detector, inspired by Ruff.

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod output;
mod sarif;
use commands::{check, version};

#[derive(Parser)]
#[command(name = "deja")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Output format (text, json, sarif)
    #[arg(short, long, global = true, default_value = "text")]
    format: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Check files for code duplication
    Check {
        /// Files or directories to check
        #[arg(required = true)]
        paths: Vec<PathBuf>,

        /// Minimum number of lines for a clone
        #[arg(long, default_value = "4")]
        min_lines: usize,

        /// Minimum number of tokens for a clone
        #[arg(long, default_value = "20")]
        min_tokens: usize,

        /// Similarity threshold (0.0 to 1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Detection mode (fast, balanced, precise)
        #[arg(long, default_value = "balanced")]
        mode: String,

        /// Exclude files with "test" in the name
        #[arg(long)]
        exclude_tests: bool,

        /// Maximum number of clone groups to display (0 = unlimited)
        #[arg(long, default_value = "0")]
        max_groups: usize,

        /// Display only summary statistics, not individual clones
        #[arg(long)]
        summary_only: bool,

        /// Show code snippets in output (same as --verbose)
        #[arg(long)]
        show_code: bool,
    },

    /// Show version information
    Version,
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Check {
            paths,
            min_lines,
            min_tokens,
            threshold,
            mode,
            exclude_tests,
            max_groups,
            summary_only,
            show_code,
        } => check::run(
            paths,
            min_lines,
            min_tokens,
            threshold,
            &mode,
            &cli.format,
            cli.verbose || show_code,
            exclude_tests,
            max_groups,
            summary_only,
        ),
        Commands::Version => {
            version::run();
            Ok(())
        }
    }
}
