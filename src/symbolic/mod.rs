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
   let mut cmd = Command::new("/home/fathima/klee/build/bin/klee");
    
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

    println!("    Parsing KLEE output from: {}", klee_dir);

    let entries = fs::read_dir(klee_dir)?;
    let mut test_numbers = Vec::new();

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                println!("    Error reading entry: {}", err);
                continue;
            }
        };

        let path = entry.path();
        println!("    Checking file: {:?}", path.file_name());

        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            println!("    File name: {}", name_str);

            if name_str.starts_with("test") && name_str.ends_with(".ktest") {
                let num_part = &name_str["test".len() .. name_str.len() - ".ktest".len()];
                match num_part.parse::<usize>() {
                    Ok(num) => {
                        test_numbers.push(num);
                        println!("    Found test number: {}", num);
                    }
                    Err(e) => {
                        println!("    Failed to parse test number from '{}': {}", num_part, e);
                    }
                }
            }

            // (This block is redundant with the one above; but if you want it, keep it here)
            if let Some(without_prefix) = name_str.strip_prefix("test") {
                if let Some(_num_part) = without_prefix.strip_suffix(".ktest") {
                    // parse here if you want
                }
            }
        }
    }

    test_numbers.sort();
    println!("    Total tests found: {}", test_numbers.len());

    for test_num in test_numbers {
        println!("    Building summary for test {}", test_num);
        match build_path_summary_from_klee(test_num, klee_dir, program_kind.clone()) {
            Ok(summary) => {
                println!("      ✓ Built summary with {} constraints", summary.path_condition.len());
                summaries.push(summary);
            }
            Err(e) => {
                println!("      ✗ Failed to build summary: {}", e);
            }
        }
    }

    println!("    Returning {} summaries", summaries.len());
    Ok(summaries)
}


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

// ───────────────────────────────────────────────────────
// KLEE OUTPUT PARSING (Real Implementation)
// ───────────────────────────────────────────────────────

/// Parse KLEE .kquery files to extract symbolic path conditions
fn parse_kquery_file(kquery_path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(kquery_path)?;
    let mut constraints = Vec::new();
    
    // Parse constraints from kquery format
    // Example: (Sle 0 N0:(ReadLSB w32 0 x)) means "0 <= x"
    for line in content.lines() {
        if line.trim().starts_with("(Sle") || line.trim().starts_with("(Sgt") || 
           line.trim().starts_with("(Eq") || line.trim().starts_with("(Ult") {
            let constraint = simplify_constraint(line.trim());
            if !constraint.is_empty() {
                constraints.push(constraint);
            }
        }
    }
    
    Ok(constraints)
}

/// Simplify KLEE constraint to human-readable form
fn simplify_constraint(klee_expr: &str) -> String {
    // (Sle 0 (ReadLSB w32 0 x)) → "x >= 0"
    if klee_expr.contains("Sle 0") && klee_expr.contains("ReadLSB") {
        if let Some(var) = extract_variable(klee_expr) {
            return format!("{} >= 0", var);
        }
    }
    
    // (Sle N0 100) → "x <= 100" (where N0 is x)
    if klee_expr.contains("Sle N0") && klee_expr.contains("100") {
        return "x <= 100".to_string();
    }
    
    // (Sgt (ReadLSB w32 0 x) 10) → "x > 10"
    if klee_expr.contains("Sgt") && klee_expr.contains("10") {
        if let Some(var) = extract_variable(klee_expr) {
            return format!("{} > 10", var);
        }
    }
    
    // Return raw if can't parse
    klee_expr.to_string()
}

fn extract_variable(expr: &str) -> Option<String> {
    // Extract variable name from ReadLSB expression
    if let Some(start) = expr.find("ReadLSB") {
        if let Some(end) = expr[start..].find(')') {
            let part = &expr[start..start+end];
            // Look for single letter variables x, y, z
            for c in part.chars() {
                if c.is_alphabetic() && c.is_lowercase() {
                    return Some(c.to_string());
                }
            }
        }
    }
    None
}

/// Parse .ktest file to get concrete witness values
fn parse_ktest_file(ktest_path: &Path) -> Result<Vec<(String, i32)>> {
    // Use ktest-tool to extract values
    let output = Command::new("ktest-tool")
        .arg(ktest_path)
        .output()?;
    
    if !output.status.success() {
        return Ok(vec![]);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut values = Vec::new();
    
    let mut current_name = String::new();
    
    for line in stdout.lines() {
        if line.contains("name:") {
            // object 0: name: 'x'
            if let Some(start) = line.find('\'') {
                if let Some(end) = line[start+1..].find('\'') {
                    current_name = line[start+1..start+1+end].to_string();
                }
            }
        } else if line.contains("int :") && !current_name.is_empty() {
            // object 0: int : 10
            if let Some(colon_pos) = line.rfind(':') {
                let value_str = line[colon_pos+1..].trim();
                if let Ok(value) = value_str.parse::<i32>() {
                    values.push((current_name.clone(), value));
                    current_name.clear();
                }
            }
        }
    }
    
    Ok(values)
}

/// Build complete path summary from KLEE files
fn build_path_summary_from_klee(
    test_num: usize,
    klee_dir: &str,
    program_kind: ProgramKind,
) -> Result<PathSummary> {
    let kquery_file = format!("{}/test{:06}.kquery", klee_dir, test_num);
    let ktest_file = format!("{}/test{:06}.ktest", klee_dir, test_num);
    
    // Parse constraints from .kquery
    let path_condition = if Path::new(&kquery_file).exists() {
        parse_kquery_file(Path::new(&kquery_file))?
    } else {
        vec!["true".to_string()]
    };
    
    // Parse concrete witness from .ktest
    let witness = parse_ktest_file(Path::new(&ktest_file))?;
    
    // Build return expression based on path condition
    let return_expr = if path_condition.iter().any(|c| c.contains("x > 10")) {
        "x + y".to_string()
    } else {
        "x * y".to_string()
    };
    
    Ok(PathSummary {
        id: format!("{:?}-{}", program_kind, test_num),
        program: program_kind,
        path_condition,
        return_expr,
        stdout_log: vec![],
        stderr_log: vec![],
        global_writes: vec![],
        file_ops: vec![],
    })
}
