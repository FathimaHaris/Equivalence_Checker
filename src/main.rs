// src/main.rs
// ═══════════════════════════════════════════════════════
// Entry point — CLI mode OR web UI mode
//   cargo run                     → shows help
//   cargo run -- --ui             → opens browser UI on :8080
//   cargo run -- --ui --port 3000 → custom port
//   cargo run -- --c-file ...     → classic CLI mode
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
mod server; 
mod diff;                       // ← new

use types::{AnalysisConfig, InputBound, Verdict};

#[derive(Parser, Debug)]
#[command(name = "equivalence_checker")]
#[command(about = "LLVM-Based Cross-Language Semantic Equivalence Checker\n\nRun with --ui to open the browser interface.")]
struct Cli {
    // ── UI mode ───────────────────────────────────────
    /// Launch the browser UI instead of CLI mode
    #[arg(long)]
    ui: bool,

    /// Port for the UI server (default: 8080)
    #[arg(long, default_value = "8080")]
    port: u16,

    // ── CLI mode ──────────────────────────────────────
    /// Path to C source file
    #[arg(long, value_name = "FILE", required_unless_present = "ui")]
    c_file: Option<String>,

    /// Path to Rust source file
    #[arg(long, value_name = "FILE", required_unless_present = "ui")]
    rust_file: Option<String>,

    /// Function name to check
    #[arg(long, value_name = "NAME", required_unless_present = "ui")]
    function: Option<String>,

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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();

    if cli.ui {
        // ── Web UI mode ───────────────────────────────
        print_banner();
        server::launch(cli.port).await?;
        return Ok(());
    }

    // ── CLI mode ──────────────────────────────────────
    // All three are required when not in UI mode (clap enforces this)
    let c_file    = cli.c_file.unwrap();
    let rust_file = cli.rust_file.unwrap();
    let function  = cli.function.unwrap();

    print_banner();

    let bounds = parse_bounds(&cli.bounds)?;
    let config = AnalysisConfig {
        c_file:        c_file.clone(),
        rust_file:     rust_file.clone(),
        function_name: function.clone(),
        bounds,
        max_paths: cli.max_paths,
        timeout:   cli.timeout,
    };

    println!("{}", "═".repeat(60).blue());
    println!("{} {}", "Analyzing function:".bold(), function.yellow());
    println!("{} {}", "C file:".bold(),    c_file.cyan());
    println!("{} {}", "Rust file:".bold(), rust_file.cyan());
    println!("{}", "═".repeat(60).blue());

    // Step 1
    println!("\n{}", "[ Step 1/7 ] Input Validation...".bold().white());
    let validation = validator::validate(&config)?;
    if !validation.success {
        for err in &validation.errors { println!("  {} {}", "✗".red(), err); }
        return Err(anyhow::anyhow!("Validation failed"));
    }
    println!("  {} Validation passed", "✓".green());

    // Step 2
    println!("\n{}", "[ Step 2/7 ] Compiling to LLVM IR...".bold().white());
    let ir_files = compiler::compile(&config)?;
    println!("  {} C IR:    {}", "✓".green(), ir_files.c_ir_path.cyan());
    println!("  {} Rust IR: {}", "✓".green(), ir_files.rust_ir_path.cyan());

    // Step 3
    println!("\n{}", "[ Step 3/7 ] Normalizing IR...".bold().white());
    let normalized = normalizer::normalize(&config, &ir_files)?;
    println!("  {} Normalization complete", "✓".green());

    // Step 4
    println!("\n{}", "[ Step 4/7 ] Instrumenting IR...".bold().white());
    let instrumented = instrumentor::instrument(&config, &normalized)?;
    println!("  {} Instrumentation complete", "✓".green());

    // Step 5
    println!("\n{}", "[ Step 5/7 ] Running Symbolic Execution (KLEE)...".bold().white());
    let summaries = symbolic::execute(&config, &instrumented)?;
    println!("  {} C paths:    {}", "✓".green(), summaries.c_summaries.len());
    println!("  {} Rust paths: {}", "✓".green(), summaries.rust_summaries.len());

    // Step 6
    println!("\n{}", "[ Step 6/7 ] Checking Equivalence...".bold().white());
    let result = equivalence::check(
        &config,
        &ir_files,
        &summaries.c_summaries,
        &summaries.rust_summaries,
    )?;

    println!("\n{}", "═".repeat(60).blue());
    match result.verdict {
        Verdict::Equivalent => {
            println!("  {} Programs are SEMANTICALLY EQUIVALENT", "✓".green().bold());
        }
        Verdict::NotEquivalent => {
            println!("  {} Programs are NOT EQUIVALENT", "✗".red().bold());
            if let Some(ce) = &result.counterexample {
                println!("  {} Counterexample found:", "→".yellow());
                for (name, val) in &ce.inputs {
                    println!("      {} = {}", name.cyan(), val);
                }
                println!("      C returned:    {}", ce.c_behavior.return_value.red());
                println!("      Rust returned: {}", ce.rust_behavior.return_value.green());
            }
        }
        Verdict::Unknown => {
            println!("  {} Could not determine equivalence", "?".yellow().bold());
        }
    }
    println!("{}", "═".repeat(60).blue());

    // Step 7
    println!("\n{}", "[ Step 7/7 ] Generating Report...".bold().white());
    let report_path = reporter::generate(&config, &result)?;
    println!("  {} Report: {}", "✓".green(), report_path.cyan());

    Ok(())
}

fn parse_bounds(s: &str) -> Result<Vec<InputBound>> {
    let mut out = Vec::new();
    for part in s.split(',') {
        let p: Vec<&str> = part.split(':').collect();
        if p.len() != 3 {
            return Err(anyhow::anyhow!("Invalid bounds '{}'. Use name:min:max", part));
        }
        out.push(InputBound {
            name: p[0].to_string(),
            min:  p[1].parse::<i64>()?,
            max:  p[2].parse::<i64>()?,
        });
    }
    Ok(out)
}

fn print_banner() {
    println!("{}", "╔══════════════════════════════════════════════════════╗".blue());
    println!("{}", "║   LLVM-Based Semantic Equivalence Checker            ║".blue());
    println!("{}", "║   C ↔ Rust Cross-Language Verification               ║".blue());
    println!("{}", "╚══════════════════════════════════════════════════════╝".blue());
    println!();
}