// src/symbolic/mod.rs
// ═══════════════════════════════════════════════════════
// Module 5: Symbolic Execution using KLEE
// Explores all paths and builds path summaries
// ═══════════════════════════════════════════════════════

use crate::types::{AnalysisConfig, PathSummary, ProgramKind, CheckerError};
use crate::instrumentor::InstrumentedFiles;
use anyhow::Result;
use std::process::Command;
use std::fs;
use std::path::Path;

/// Output from symbolic execution
#[derive(Debug, Clone)]
pub struct SymbolicSummaries {
    pub c_summaries:    Vec<PathSummary>,
    pub rust_summaries: Vec<PathSummary>,
}

/// Main symbolic execution entry point
pub fn execute(config: &AnalysisConfig, files: &InstrumentedFiles) -> Result<SymbolicSummaries> {
    // Step 1: Run KLEE on C program
    println!("  Running KLEE on C IR...");
    let c_summaries = run_klee(
        &files.c_instrumented_path,
        &config.function_name,
        &config.bounds,
        config.max_paths,
        config.timeout,
        ProgramKind::C
    )?;
    println!("    → Found {} paths", c_summaries.len());

    // Step 2: Run KLEE on Rust program
    println!("  Running KLEE on Rust IR...");
    let rust_summaries = run_klee(
        &files.rust_instrumented_path,
        &config.function_name,
        &config.bounds,
        config.max_paths,
        config.timeout,
        ProgramKind::Rust
    )?;
    println!("    → Found {} paths", rust_summaries.len());

    Ok(SymbolicSummaries {
        c_summaries,
        rust_summaries,
    })
}

// ───────────────────────────────────────────────────────
// KLEE EXECUTION
// ───────────────────────────────────────────────────────

/// Run KLEE on a single IR file
/// Run KLEE on a single IR file
fn run_klee(
    ir_path: &str,
    function_name: &str,
    _bounds: &[crate::types::InputBound],
    _max_paths: u32,
    timeout: u32,
    program_kind: ProgramKind,
) -> Result<Vec<PathSummary>> {
    
    // Create KLEE output directory
    let klee_out_dir = format!("/tmp/equivalence_checker/klee_{}_{:?}", 
        function_name, program_kind);
    
    // Remove old output if exists
    if Path::new(&klee_out_dir).exists() {
        let _ = fs::remove_dir_all(&klee_out_dir);
    }

    // Build KLEE command
    let mut cmd = Command::new("klee");
    
    // Output directory
    cmd.arg("--output-dir").arg(&klee_out_dir);
    
    // Search heuristics
    cmd.arg("--search=dfs");  // Depth-first search
    
    // Limits
    cmd.arg(format!("--max-time={}", timeout));
    
    // Simplify constraints
    cmd.arg("--simplify-sym-indices");
    
    // Generate test cases
    cmd.arg("--write-test-info");
    cmd.arg("--write-paths");
    
    // Input bitcode file
    cmd.arg(ir_path);
    
    // Entry point (if specified)
    // if !function_name.is_empty() {
    //     cmd.arg("--entry-point").arg(function_name);
    // }

    println!("    Running KLEE (this may take up to {} seconds)...", timeout);
    
    // Execute KLEE
    let output = cmd.output()?;

    // Check if KLEE ran (it's okay if it found errors, we just need output)
    if !Path::new(&klee_out_dir).exists() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckerError::SymbolicExecutionError(
            format!("KLEE failed to create output directory:\n{}", stderr)
        ).into());
    }

    // Parse KLEE output to extract path summaries
    let summaries = parse_klee_output(&klee_out_dir, program_kind.clone())?;

    // If no paths found, create a simple default path
    if summaries.is_empty() {
        println!("    (No paths found - creating default summary)");
        return Ok(vec![create_default_summary(function_name, program_kind)]);
    }

    Ok(summaries)
}
// ───────────────────────────────────────────────────────
// KLEE OUTPUT PARSING
// ───────────────────────────────────────────────────────

/// Parse KLEE output directory to extract path summaries
fn parse_klee_output(klee_dir: &str, program_kind: ProgramKind) -> Result<Vec<PathSummary>> {
    let mut summaries = Vec::new();

    // Look for test*.ktest files
    let entries = fs::read_dir(klee_dir)?;
    
    let mut test_count = 0;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            
            // Look for test files
            if name_str.starts_with("test") && name_str.ends_with(".ktest") {
                test_count += 1;
                
                // For now, create a simplified path summary
                // A real implementation would parse the .ktest file properly
                summaries.push(PathSummary {
                    id: format!("{:?}-{}", program_kind, test_count),
                    program: program_kind.clone(),
                    path_condition: vec![format!("path_{}", test_count)],
                    return_expr: "symbolic_return".to_string(),
                    stdout_log: vec![],
                    stderr_log: vec![],
                    global_writes: vec![],
                    file_ops: vec![],
                });
            }
        }
    }

    Ok(summaries)
}

/// Create a default path summary when KLEE finds no paths
fn create_default_summary(function_name: &str, program_kind: ProgramKind) -> PathSummary {
    PathSummary {
        id: format!("{:?}-default", program_kind),
        program: program_kind,
        path_condition: vec!["true".to_string()],
        return_expr: format!("{}(...)", function_name),
        stdout_log: vec![],
        stderr_log: vec![],
        global_writes: vec![],
        file_ops: vec![],
    }
}

// ───────────────────────────────────────────────────────
// HELPER: Display KLEE statistics
// ───────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn display_klee_stats(klee_dir: &str) -> Result<()> {
    // Look for run.stats file
    let stats_path = format!("{}/run.stats", klee_dir);
    
    if Path::new(&stats_path).exists() {
        let content = fs::read_to_string(&stats_path)?;
        println!("\nKLEE Statistics:");
        println!("{}", content);
    }

    Ok(())
}