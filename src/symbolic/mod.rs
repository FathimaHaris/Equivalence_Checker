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
    max_paths: u32,
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

    cmd.arg("--optimize=false");
    
    // Search heuristics
    // cmd.arg("--search=dfs");  // Depth-first search

    cmd.arg("--search=random-path");  // Better than DFS for coverage
    cmd.arg("--search=nurs:covnew");   // Coverage-optimized search

    
    // Limits
    cmd.arg(format!("--max-time={}", timeout));

    cmd.arg(format!("--max-tests={}", max_paths));
    
    // Simplify constraints
    cmd.arg("--simplify-sym-indices");
    
    // Generate test cases
    cmd.arg("--write-test-info");
    cmd.arg("--write-paths");
    cmd.arg("--write-kqueries");



    cmd.arg("--max-memory=1000");  // Limit memory
    cmd.arg("--only-output-states-covering-new");
    


    match program_kind {
    ProgramKind::C => {
        cmd.arg("--entry-point=main");
    }
    ProgramKind::Rust => {
        cmd.arg("--entry-point=klee_harness");
    }
    }

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

    // // If no paths found, create a simple default path
    // if summaries.is_empty() {
    //     println!("    (No paths found - creating default summary)");
    //     return Ok(vec![create_default_summary(function_name, program_kind)]);
    // }

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


// fn create_default_summary(function_name: &str, program_kind: ProgramKind) -> PathSummary {
//     PathSummary {
//         id: format!("{:?}-default", program_kind),
//         program: program_kind,
//         path_condition: vec!["true".to_string()],
//         return_expr: format!("{}(...)", function_name),
//         stdout_log: vec![],
//         stderr_log: vec![],
//         global_writes: vec![],
//         file_ops: vec![],
            // witness: vec![],
//     }
// }

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
// fn parse_kquery_file(kquery_path: &Path) -> Result<Vec<String>> {
//     let content = fs::read_to_string(kquery_path)?;
//     let mut constraints = Vec::new();
    
//     // Parse constraints from kquery format
//     // Example: (Sle 0 N0:(ReadLSB w32 0 x)) means "0 <= x"
//     for line in content.lines() {
//         if line.trim().starts_with("(Sle") || line.trim().starts_with("(Sgt") || 
//            line.trim().starts_with("(Eq") || line.trim().starts_with("(Ult") {
//             let constraint = simplify_constraint(line.trim());
//             if !constraint.is_empty() {
//                 constraints.push(constraint);
//             }
//         }
//     }
    
//     Ok(constraints)
// }



fn parse_ktest_file(ktest_path: &Path) -> Result<Vec<(String, i32)>> {
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
                if let Some(end) = line[start + 1..].find('\'') {
                    current_name = line[start + 1..start + 1 + end].to_string();
                }
            }
        } else if line.contains("int :") && !current_name.is_empty() {
            // object 0: int : 10
            if let Some(colon_pos) = line.rfind(':') {
                let value_str = line[colon_pos + 1..].trim();
                if let Ok(value) = value_str.parse::<i32>() {
                    values.push((current_name.clone(), value));
                    current_name.clear();
                }
            }
        }
    }

    Ok(values)
}


fn parse_kquery_file(kquery_path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(kquery_path)?;

    // Find the constraints list inside: (query [ ... ] false)
    let start = content.find("(query [")
        .ok_or_else(|| anyhow::anyhow!("kquery missing '(query ['"))?;
    let after = &content[start + "(query [".len()..];

    let end = after.find("]")  // end of the constraint list
        .ok_or_else(|| anyhow::anyhow!("kquery missing closing ']'"))?;
    let list = &after[..end];

    // Now parse balanced (...) expressions from `list`
    let mut constraints = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();

    for ch in list.chars() {
        if ch == '(' {
            if depth == 0 {
                cur.clear();
            }
            depth += 1;
        }

        if depth > 0 {
            cur.push(ch);
        }

        if ch == ')' && depth > 0 {
            depth -= 1;
            if depth == 0 {
                constraints.push(cur.trim().to_string());
            }
        }
    }

    if constraints.is_empty() {
        // If there are truly no constraints, treat as true
        constraints.push("true".to_string());
    }

    Ok(constraints)
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
    let witness_i32 = parse_ktest_file(Path::new(&ktest_file))?;
    let witness: Vec<(String, i64)> = witness_i32
        .into_iter()
        .map(|(n, v)| (n, v as i64))
        .collect();
    println!("      witness: {:?}", witness);
    
    // For now, we don't infer symbolic return.
    // It will be handled in the equivalence module.
    // Change this line in build_path_summary_from_klee:
    // let return_expr = extract_return_expr_from_klee(Path::new(&kquery_file))?;
    // // Or convert String to Path:
    // let return_expr = extract_return_expr_from_klee(kquery_file.as_ref())?;


    let return_expr = if Path::new(&kquery_file).exists() {
        extract_return_expr_from_klee(Path::new(&kquery_file))?
    } else {
        "UNKNOWN".to_string()
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
        witness,
    })
}


/// Extract return expression from KLEE query file
fn extract_return_expr_from_klee(kquery_path: &Path) -> Result<String> {
    let content = fs::read_to_string(kquery_path)?;
    
    // Look for the query expression that represents the return value
    // This is a simplified approach - you'll need to adapt based on your KLEE output
    
    // Common patterns:
    // 1. Look for return variable (often named "result" or the function return)
    if content.contains("result") {
        // Try to find expression for result
        if let Some(expr) = extract_expr_for_var(&content, "result") {
            return Ok(expr);
        }
    }
    
    // 2. Look for the last expression before false
    if let Some(query_start) = content.find("(query [") {
        let after_query = &content[query_start + 7..];
        if let Some(expr_end) = after_query.find(" false") {
            let expr_part = &after_query[..expr_end];
            // Extract the last expression which might be the return
            if let Some(last_expr) = extract_last_expression(expr_part) {
                return Ok(last_expr);
            }
        }
    }
    
    // Fallback: return UNKNOWN but with a warning
    println!("      Warning: Could not extract return expression from KLEE");
    Ok("UNKNOWN".to_string())
}

fn extract_expr_for_var(content: &str, var_name: &str) -> Option<String> {
    // Look for patterns like: (= result (+ x y))
    let pattern = format!("(= {} ", var_name);
    if let Some(start) = content.find(&pattern) {
        let start = start + pattern.len();
        let mut depth = 1;
        let mut end = start;
        
        for (i, c) in content[start..].chars().enumerate() {
            if c == '(' { depth += 1; }
            else if c == ')' { 
                depth -= 1;
                if depth == 0 {
                    end = start + i;
                    break;
                }
            }
        }
        
        if end > start {
            return Some(content[start..=end].to_string());
        }
    }
    None
}

fn extract_last_expression(expr_part: &str) -> Option<String> {
    let mut depth = 0;
    let mut last_expr_start = 0;
    
    for (i, c) in expr_part.char_indices() {
        if c == '(' {
            if depth == 0 {
                last_expr_start = i;
            }
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                // Found a complete expression
                last_expr_start = i + 1; // Start after this one
            }
        }
    }
    
    if last_expr_start < expr_part.len() {
        Some(expr_part[last_expr_start..].trim().to_string())
    } else {
        None
    }
}