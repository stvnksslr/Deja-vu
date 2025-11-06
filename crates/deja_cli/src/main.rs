//! Deja-vu CLI - Code duplication detector
//!
//! An extremely fast code duplication detector, inspired by Ruff.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::path::PathBuf;

mod commands;
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
        #[arg(long, default_value = "5")]
        min_lines: usize,

        /// Minimum number of tokens for a clone
        #[arg(long, default_value = "50")]
        min_tokens: usize,

        /// Similarity threshold (0.0 to 1.0)
        #[arg(long, default_value = "0.85")]
        threshold: f64,

        /// Detection mode (fast, balanced, precise)
        #[arg(long, default_value = "balanced")]
        mode: String,
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
        } => {
            check::run(paths, min_lines, min_tokens, threshold, &mode, &cli.format, cli.verbose)
        }
        Commands::Version => {
            version::run();
            Ok(())
        }
    }
}
