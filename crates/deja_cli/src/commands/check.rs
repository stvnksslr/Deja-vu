//! Check command implementation

use crate::output::JsonOutput;
use crate::sarif::SarifReport;
use anyhow::Result;
use colored::*;
use deja_core::{
    collect_files, file_statistics, CloneDetector, DetectionConfig, DetectionMode,
    TokenBasedDetector,
};
use deja_python::PythonTokenizer;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

pub fn run(
    paths: Vec<PathBuf>,
    min_lines: usize,
    min_tokens: usize,
    threshold: f64,
    mode: &str,
    format: &str,
    verbose: bool,
    exclude_tests: bool,
    max_groups: usize,
    summary_only: bool,
) -> Result<()> {
    println!(
        "{}",
        "Deja-vu: Code Duplication Detector".bright_blue().bold()
    );
    println!();

    // Parse detection mode
    let detection_mode = match mode {
        "fast" => DetectionMode::Fast,
        "balanced" => DetectionMode::Balanced,
        "precise" => DetectionMode::Precise,
        _ => {
            eprintln!(
                "{} Invalid mode '{}', using 'balanced'",
                "Warning:".yellow(),
                mode
            );
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
    let has_clones = !clone_groups.is_empty();

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

        if !summary_only {
            // Determine how many groups to display
            let display_limit = if max_groups == 0 {
                clone_groups.len()
            } else {
                max_groups.min(clone_groups.len())
            };

            for (i, group) in clone_groups.iter().enumerate().take(display_limit) {
                println!(
                    "Clone Group #{} ({}, similarity: {:.1}%)",
                    i + 1,
                    group.clone_type.as_str(),
                    group.similarity * 100.0
                );
                println!(
                    "  {} instances, avg {} lines",
                    group.size(),
                    group.avg_lines() as usize
                );

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

            if max_groups > 0 && clone_groups.len() > max_groups {
                println!(
                    "{}",
                    format!(
                        "... and {} more clone group(s) (use --max-groups 0 to show all)",
                        clone_groups.len() - max_groups
                    )
                    .yellow()
                );
                println!();
            }
        }

        let total_instances: usize = clone_groups.iter().map(|g| g.size()).sum();
        let total_duplicated_lines: usize = clone_groups
            .iter()
            .flat_map(|g| &g.instances)
            .map(|c| c.end_line - c.start_line + 1)
            .sum();

        println!(
            "{}",
            "Summary Statistics:".bright_white().bold()
        );
        println!(
            "  Total clone instances: {}",
            total_instances.to_string().red().bold()
        );
        println!(
            "  Total clone groups: {}",
            clone_groups.len().to_string().red().bold()
        );
        println!(
            "  Duplicated lines: {}",
            total_duplicated_lines.to_string().red().bold()
        );
    }

    println!();

    // Export to other formats
    match format {
        "json" => {
            let total_lines: usize = files.iter().map(|f| f.content.lines().count()).sum();
            let output = JsonOutput::new(
                clone_groups,
                &config,
                files.len(),
                total_lines,
                duration.as_secs_f64(),
            );
            let json_str = output.to_json()?;
            println!("{}", json_str);
        }
        "sarif" => {
            let report = SarifReport::from_clone_groups(clone_groups, &config);
            let sarif_str = report.to_json()?;
            println!("{}", sarif_str);
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

    // Set exit code based on results
    // Exit codes:
    // 0 = No duplicates found (success)
    // 1 = Duplicates found
    // 2 = Error (handled by anyhow's ? operator in main)
    if has_clones {
        process::exit(1);
    }

    Ok(())
}
