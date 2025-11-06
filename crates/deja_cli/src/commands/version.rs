//! Version command implementation

use colored::*;

pub fn run() {
    println!(
        "{} v{}",
        "Deja-vu".bright_blue().bold(),
        env!("CARGO_PKG_VERSION")
    );
    println!("An extremely fast code duplication detector");
    println!();
    println!("Written in Rust, inspired by Ruff");
}
