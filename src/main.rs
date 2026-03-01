// src/main.rs
// ═══════════════════════════════════════════════════════
// Entry point — handles CLI and calls each module
// ═══════════════════════════════════════════════════════

use clap::Parser;
use colored::*;
use anyhow::Result;

mod types;
mod validator;
mod compiler;
mod normalizer;
mod instrumentor;
mod symbolic;
mod equivalence;
mod reporter;

use types::{AnalysisConfig, InputBound, Verdict};

/// LLVM-Based Cross-Language Semantic Equivalence Checker
#[derive(Parser, Debug)]
#[command(name = "equivalence_checker")]
#[command(about = "Checks semantic equivalence between C and Rust programs")]
struct Cli {
    /// Path to C source file
    #[arg(long, value_name = "FILE")]
    c_file: String,

    /// Path to Rust source file
    #[arg(long, value_name = "FILE")]
    rust_file: String,

    /// Function name to check
    #[arg(long, value_name = "NAME")]
    function: String,

    /// Input bounds e.g. "x:0:100,y:0:100"
    #[arg(long, value_name = "BOUNDS", default_value = "x:0:100")]
    bounds: String,

    /// Maximum paths to explore
    #[arg(long, default_value = "100")]
    max_paths: u32,

    /// Timeout in seconds
    #[arg(long, default_value = "60")]
    timeout: u32,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    print_banner();

    let bounds = parse_bounds(&cli.bounds)?;
    let config = AnalysisConfig {
        c_file:        cli.c_file.clone(),
        rust_file:     cli.rust_file.clone(),
        function_name: cli.function.clone(),
        bounds,
        max_paths:     cli.max_paths,
        timeout:       cli.timeout,
    };

    println!("{}", "═".repeat(60).blue());
    println!("{} {}", "Analyzing function:".bold(), cli.function.yellow());
    println!("{} {}", "C file:".bold(),    cli.c_file.cyan());
    println!("{} {}", "Rust file:".bold(), cli.rust_file.cyan());
    println!("{}", "═".repeat(60).blue());

    // ── Step 1: Validate ─────────────────────────────────
    println!("\n{}", "[ Step 1/7 ] Input Validation...".bold().white());
    let validation = validator::validate(&config)?;
    if !validation.success {
        for err in &validation.errors {
            println!("  {} {}", "✗".red(), err);
        }
        return Err(anyhow::anyhow!("Validation failed"));
    }
    println!("  {} Validation passed", "✓".green());

    // ── Step 2: Compile ──────────────────────────────────
    println!("\n{}", "[ Step 2/7 ] Compiling to LLVM IR...".bold().white());
    let ir_files = compiler::compile(&config)?;
    println!("  {} C IR:    {}", "✓".green(), ir_files.c_ir_path.cyan());
    println!("  {} Rust IR: {}", "✓".green(), ir_files.rust_ir_path.cyan());
    println!("  {} C runner:    {}", "✓".green(), ir_files.c_runner_bin.cyan());
    println!("  {} Rust runner: {}", "✓".green(), ir_files.rust_runner_bin.cyan());

    // ── Step 3: Normalize ────────────────────────────────
    println!("\n{}", "[ Step 3/7 ] Normalizing IR...".bold().white());
    let normalized = normalizer::normalize(&config, &ir_files)?;
    println!("  {} Normalization complete", "✓".green());

    // ── Step 4: Instrument ───────────────────────────────
    println!("\n{}", "[ Step 4/7 ] Instrumenting IR...".bold().white());
    let instrumented = instrumentor::instrument(&config, &normalized)?;
    println!("  {} Instrumentation complete", "✓".green());

    // ── Step 5: Symbolic Execution ───────────────────────
    println!("\n{}", "[ Step 5/7 ] Running Symbolic Execution (KLEE)...".bold().white());
    let summaries = symbolic::execute(&config, &instrumented)?;
    println!("  {} C paths found:    {}", "✓".green(), summaries.c_summaries.len());
    println!("  {} Rust paths found: {}", "✓".green(), summaries.rust_summaries.len());

    // ── Step 6: Equivalence Checking ─────────────────────
    println!("\n{}", "[ Step 6/7 ] Checking Equivalence...".bold().white());
    let result = equivalence::check(
        &config,
        &ir_files,                    // ← pass runner binaries
        &summaries.c_summaries,
        &summaries.rust_summaries,
    )?;

    // ── Print verdict ────────────────────────────────────
    println!("\n{}", "═".repeat(60).blue());
    match result.verdict {
        Verdict::Equivalent => {
            println!(
                "  {} Programs are SEMANTICALLY EQUIVALENT",
                "✓".green().bold()
            );
        }
        Verdict::NotEquivalent => {
            println!("  {} Programs are NOT EQUIVALENT", "✗".red().bold());
            if let Some(ce) = &result.counterexample {
                println!("  {} Counterexample found:", "→".yellow());
                for (name, val) in &ce.inputs {
                    println!("      {} = {}", name.cyan(), val);
                }
                println!(
                    "      C returned:    {}",
                    ce.c_behavior.return_value.red()
                );
                println!(
                    "      Rust returned: {}",
                    ce.rust_behavior.return_value.green()
                );
            }
        }
        Verdict::Unknown => {
            println!(
                "  {} Could not determine equivalence (timeout/unknown)",
                "?".yellow().bold()
            );
        }
    }
    println!("{}", "═".repeat(60).blue());

    // ── Step 7: Report ───────────────────────────────────
    println!("\n{}", "[ Step 7/7 ] Generating Report...".bold().white());
    let report_path = reporter::generate(&config, &result)?;
    println!("  {} Report saved to: {}", "✓".green(), report_path.cyan());

    Ok(())
}

fn parse_bounds(bounds_str: &str) -> Result<Vec<InputBound>> {
    let mut bounds = Vec::new();
    for part in bounds_str.split(',') {
        let parts: Vec<&str> = part.split(':').collect();
        if parts.len() != 3 {
            return Err(anyhow::anyhow!(
                "Invalid bounds format '{}'. Use: name:min:max", part
            ));
        }
        bounds.push(InputBound {
            name: parts[0].to_string(),
            min:  parts[1].parse::<i64>()?,
            max:  parts[2].parse::<i64>()?,
        });
    }
    Ok(bounds)
}

fn print_banner() {
    println!("{}", "╔══════════════════════════════════════════════════════╗".blue());
    println!("{}", "║   LLVM-Based Semantic Equivalence Checker            ║".blue());
    println!("{}", "║   C ↔ Rust Cross-Language Verification               ║".blue());
    println!("{}", "╚══════════════════════════════════════════════════════╝".blue());
    println!();
}