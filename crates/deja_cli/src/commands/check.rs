//! Check command implementation

use anyhow::Result;
use colored::*;
use deja_core::{
    collect_files, file_statistics, CloneDetector, DetectionConfig, DetectionMode,
    TokenBasedDetector,
};
use deja_python::PythonTokenizer;
use std::path::PathBuf;
use std::time::Instant;
use serde_json;

pub fn run(
    paths: Vec<PathBuf>,
    min_lines: usize,
    min_tokens: usize,
    threshold: f64,
    mode: &str,
    format: &str,
    verbose: bool,
    exclude_tests: bool,
) -> Result<()> {
    println!("{}", "Deja-vu: Code Duplication Detector".bright_blue().bold());
    println!();

    // Parse detection mode
    let detection_mode = match mode {
        "fast" => DetectionMode::Fast,
        "balanced" => DetectionMode::Balanced,
        "precise" => DetectionMode::Precise,
        _ => {
            eprintln!("{} Invalid mode '{}', using 'balanced'", "Warning:".yellow(), mode);
            DetectionMode::Balanced
        }
    };

    // Create configuration
    let config = DetectionConfig {
        mode: detection_mode,
        min_lines,
        min_tokens,
        similarity_threshold: threshold,
        ignore_comments: true,
        ignore_whitespace: true,
    };

    if verbose {
        println!("Configuration:");
        println!("  Mode: {:?}", config.mode);
        println!("  Min lines: {}", config.min_lines);
        println!("  Min tokens: {}", config.min_tokens);
        println!("  Threshold: {:.2}", config.similarity_threshold);
        println!("  Format: {}", format);
        println!();
    }

    // Collect files
    println!("Collecting files from {} path(s)...", paths.len());
    let start = Instant::now();

    // For now, only support Python files
    let files = collect_files(&paths, &["py"], exclude_tests)?;

    if files.is_empty() {
        println!("{}", "No Python files found!".yellow());
        return Ok(());
    }

    let stats = file_statistics(&files);
    println!("  {}", stats.summary().green());
    println!("  Collected in {:.2}s", start.elapsed().as_secs_f64());
    println!();

    // Create detector with Python tokenizer
    let mut detector = TokenBasedDetector::new();
    detector.register_tokenizer("python".to_string(), Box::new(PythonTokenizer::new()));

    // Run detection
    println!("Analyzing code for duplicates...");
    let detect_start = Instant::now();

    let clone_groups = detector.detect(&files, &config)?;

    let duration = detect_start.elapsed();

    // Display results
    println!();
    if clone_groups.is_empty() {
        println!("{}", "✓ No code duplicates found!".green().bold());
    } else {
        println!(
            "{}",
            format!("Found {} clone group(s)", clone_groups.len())
                .red()
                .bold()
        );
        println!();

        for (i, group) in clone_groups.iter().enumerate() {
            println!(
                "Clone Group #{} ({}, similarity: {:.1}%)",
                i + 1,
                group.clone_type.as_str(),
                group.similarity * 100.0
            );
            println!("  {} instances, avg {} lines", group.size(), group.avg_lines() as usize);

            for instance in &group.instances {
                println!(
                    "    {}:{}:{}-{}:{}",
                    instance.file.display().to_string().cyan(),
                    instance.start_line,
                    instance.start_col,
                    instance.end_line,
                    instance.end_col
                );

                if verbose {
                    // Show complete code content
                    for line in instance.content.lines() {
                        println!("      {}", line.dimmed());
                    }
                }
            }
            println!();
        }

        let total_instances: usize = clone_groups.iter().map(|g| g.size()).sum();
        println!(
            "Total: {} clone instances in {} groups",
            total_instances.to_string().red().bold(),
            clone_groups.len().to_string().red().bold()
        );
    }

    println!();
    println!(
        "Analyzed in {:.2}s ({} files/sec)",
        duration.as_secs_f64(),
        (files.len() as f64 / duration.as_secs_f64()) as usize
    );

    // Export to other formats
    match format {
        "json" => {
            let json_output = serde_json::to_string_pretty(&clone_groups)?;
            println!("{}", json_output);
        }
        "sarif" => {
            println!(
                "{}",
                "⚠ Export format 'sarif' not yet implemented".yellow()
            );
        }
        "text" => {
            // Already displayed above
        }
        _ => {
            println!(
                "{}",
                format!("⚠ Unknown export format '{}'", format).yellow()
            );
        }
    }

    Ok(())
}
