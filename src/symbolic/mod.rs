// src/symbolic/mod.rs
// ═══════════════════════════════════════════════════════
// Module 5: Symbolic Execution using KLEE
// Steps: 0.5.1 Symbolic Inputs → 0.5.2 Path Exploration →
//        0.5.3 Constraint Extraction → 0.5.4 Path Summary Construction
// ═══════════════════════════════════════════════════════

use crate::types::{
    AnalysisConfig, PathSummary, ProgramKind, CheckerError,
    ObservableEffects,
};
use anyhow::Result;
use std::process::Command;
use std::fs;
use std::path::Path;

/// Output from symbolic execution
#[derive(Debug, Clone)]
pub struct SymbolicSummaries {
    pub c_summaries: Vec<PathSummary>,
    pub rust_summaries: Vec<PathSummary>,
}

/// Main symbolic execution entry point
pub fn execute(
    config: &AnalysisConfig,
    files: &crate::instrumentor::InstrumentedFiles,
) -> Result<SymbolicSummaries> {
    println!("  Running KLEE on C IR...");
    let c_summaries = run_klee(
        &files.c_instrumented_path,
        &config.function_name,
        config.max_paths,
        config.timeout,
        ProgramKind::C,
    )?;
    println!("    → Found {} paths", c_summaries.len());

    println!("  Running KLEE on Rust IR...");
    let rust_summaries = run_klee(
        &files.rust_instrumented_path,
        &config.function_name,
        config.max_paths,
        config.timeout,
        ProgramKind::Rust,
    )?;
    println!("    → Found {} paths", rust_summaries.len());

    // If KLEE found very few paths (likely missed branches), warn the user
    let expected_min = 1usize;
    if c_summaries.len() <= expected_min || rust_summaries.len() <= expected_min {
        println!("    ⚠ Warning: KLEE may have missed some paths.");
        println!("      C paths: {}, Rust paths: {}", c_summaries.len(), rust_summaries.len());
        println!("      The equivalence checker will use concrete testing as a fallback.");
    }

    Ok(SymbolicSummaries {
        c_summaries,
        rust_summaries,
    })
}

// ───────────────────────────────────────────────────────
// Step 0.5.1 & 0.5.2: KLEE Execution
// ───────────────────────────────────────────────────────

fn run_klee(
    ir_path: &str,
    function_name: &str,
    _max_paths: u32,
    timeout: u32,
    program_kind: ProgramKind,
) -> Result<Vec<PathSummary>> {
    let kind_str = match program_kind {
        ProgramKind::C => "C",
        ProgramKind::Rust => "Rust",
    };

    let klee_out_dir = format!(
        "/tmp/equivalence_checker/klee_{}_{}",
        function_name, kind_str
    );

    if Path::new(&klee_out_dir).exists() {
        let _ = fs::remove_dir_all(&klee_out_dir);
    }

    let mut cmd = Command::new("/home/fathima/klee/build/bin/klee");
    cmd.arg("--output-dir").arg(&klee_out_dir);
    cmd.arg("--optimize=false");
    cmd.arg("--search=dfs");
    cmd.arg("--max-forks=500");
    cmd.arg("--max-depth=500");
    cmd.arg("--max-tests=200");
    cmd.arg(format!("--max-time={}", timeout));
    cmd.arg("--simplify-sym-indices");
    cmd.arg("--write-test-info");
    cmd.arg("--write-paths");
    cmd.arg("--write-kqueries");
    cmd.arg("--max-memory=1000");

    // ── Entry point ────────────────────────────────────
    match program_kind {
        ProgramKind::C => {
            cmd.arg("--entry-point=main");
        }
        ProgramKind::Rust => {
            // klee_harness is exported as #[no_mangle] pub extern "C"
            cmd.arg("--entry-point=klee_harness");
        }
    }

    cmd.arg(ir_path);

    println!("    Running KLEE (this may take up to {} seconds)...", timeout);
    let output = cmd.output()?;

    // Print KLEE stderr for debugging
    let klee_stderr = String::from_utf8_lossy(&output.stderr);
    if !klee_stderr.is_empty() {
        // Only print warnings/errors, skip verbose lines
        for line in klee_stderr.lines() {
            if line.contains("ERROR") || line.contains("WARNING") || line.contains("KLEE:") {
                println!("    [KLEE] {}", line);
            }
        }
    }

    if !Path::new(&klee_out_dir).exists() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckerError::SymbolicExecutionError(format!(
            "KLEE failed to create output directory:\n{}",
            stderr
        ))
        .into());
    }

    // Step 0.5.3 & 0.5.4: Parse output into path summaries
    let summaries = parse_klee_output(&klee_out_dir, program_kind)?;
    Ok(summaries)
}

// ───────────────────────────────────────────────────────
// Step 0.5.3: Parse KLEE output directory
// ───────────────────────────────────────────────────────

fn parse_klee_output(klee_dir: &str, program_kind: ProgramKind) -> Result<Vec<PathSummary>> {
    let mut summaries = Vec::new();
    let mut test_numbers = Vec::new();

    let entries = fs::read_dir(klee_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with("test") && name_str.ends_with(".ktest") {
                let num_part =
                    &name_str["test".len()..name_str.len() - ".ktest".len()];
                if let Ok(num) = num_part.parse::<usize>() {
                    test_numbers.push(num);
                }
            }
        }
    }

    test_numbers.sort();
    println!("    Found {} test cases", test_numbers.len());

    for test_num in test_numbers {
        match build_path_summary(test_num, klee_dir, &program_kind) {
            Ok(summary) => summaries.push(summary),
            Err(e) => {
                println!(
                    "      Warning: Failed to build summary for test {}: {}",
                    test_num, e
                );
            }
        }
    }

    // If KLEE produced no usable summaries, create a placeholder
    // so the pipeline can still continue with concrete testing
    if summaries.is_empty() {
        println!("    ⚠ No summaries built from KLEE output — using placeholder");
        summaries.push(PathSummary {
            id: format!("{:?}-placeholder", program_kind),
            program: program_kind,
            constraints: vec!["true".to_string()],
            return_expr: None,
            witness: vec![],
            observables: ObservableEffects::default(),
        });
    }

    Ok(summaries)
}

// ───────────────────────────────────────────────────────
// Step 0.5.4: Build PathSummary from KLEE files
// ───────────────────────────────────────────────────────

fn build_path_summary(
    test_num: usize,
    klee_dir: &str,
    program_kind: &ProgramKind,
) -> Result<PathSummary> {
    let kquery_path = format!("{}/test{:06}.kquery", klee_dir, test_num);
    let ktest_path = format!("{}/test{:06}.ktest", klee_dir, test_num);

    // Step 0.5.3: Constraints from .kquery
    let constraints = if Path::new(&kquery_path).exists() {
        parse_kquery_file(Path::new(&kquery_path))?
    } else {
        vec!["true".to_string()]
    };

    // Return expression
    let return_expr = if Path::new(&kquery_path).exists() {
        extract_return_expr_from_kquery(Path::new(&kquery_path))?
    } else {
        None
    };

    // Concrete witness from .ktest
    let witness = if Path::new(&ktest_path).exists() {
        parse_ktest_file(Path::new(&ktest_path))?
    } else {
        vec![]
    };

    println!(
        "      Test {:06}: {} constraints, witness={:?}, return={:?}",
        test_num,
        constraints.len(),
        witness,
        return_expr
    );

    Ok(PathSummary {
        id: format!("{:?}-{}", program_kind, test_num),
        program: program_kind.clone(),
        constraints,
        return_expr,
        witness,
        observables: ObservableEffects::default(),
    })
}

// ───────────────────────────────────────────────────────
// .kquery Parser
// ───────────────────────────────────────────────────────

/// Parse KLEE .kquery file to extract path constraints
/// KLEE .kquery format: (query [ ...constraints... ] false)
fn parse_kquery_file(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    let mut constraints = Vec::new();

    // Find constraint list between '[' and ']' inside "(query ["
    if let Some(start_pos) = content.find("(query [") {
        let after = &content[start_pos + 8..];
        if let Some(end_pos) = find_matching_bracket(after) {
            let constraint_section = &after[..end_pos];

            // Parse top-level S-expressions
            let mut depth = 0usize;
            let mut current = String::new();

            for c in constraint_section.chars() {
                match c {
                    '(' => {
                        if depth == 0 {
                            current.clear();
                        }
                        depth += 1;
                        current.push(c);
                    }
                    ')' => {
                        if depth > 0 {
                            depth -= 1;
                            current.push(c);
                            if depth == 0 {
                                let trimmed = current.trim().to_string();
                                if !trimmed.is_empty() {
                                    constraints.push(trimmed);
                                }
                                current.clear();
                            }
                        }
                    }
                    _ if depth > 0 => current.push(c),
                    _ => {} // whitespace between top-level expressions
                }
            }
        }
    }

    if constraints.is_empty() {
        constraints.push("true".to_string());
    }

    Ok(constraints)
}

/// Find the position of the matching ']' for the '[' we just passed
fn find_matching_bracket(s: &str) -> Option<usize> {
    // We are already past '[', find the matching ']' at depth 0
    // (respecting nested parens inside, but brackets are not nested in kquery)
    let mut paren_depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            ']' if paren_depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Extract the return value expression from a .kquery file.
///
/// KLEE encodes the return/result as the *query expression* — the part
/// after `] ` and before the closing `)` of the outer `(query …)` call.
/// For a path that returns a constant the expression is something like
/// `(w32 2)` or just `false` (meaning the constraint set is the focus,
/// not the expression).  We capture whatever is there so the equivalence
/// checker can compare it symbolically.
fn extract_return_expr_from_kquery(path: &Path) -> Result<Option<String>> {
    let content = fs::read_to_string(path)?;

    // Pattern: (query [ constraints ] EXPR )
    // We want EXPR — everything after the closing ']' up to the final ')'
    if let Some(start_pos) = content.find("(query [") {
        let after = &content[start_pos + 8..];
        if let Some(bracket_end) = find_matching_bracket(after) {
            // after bracket_end+1 we have whitespace then the expression
            let rest = after[bracket_end + 1..].trim();
            // rest starts with the expression; grab the first token/s-expr
            let expr = extract_first_token_or_sexpr(rest);
            if !expr.is_empty() && expr != "false" && expr != ")" {
                return Ok(Some(expr));
            }
        }
    }
    Ok(None)
}

/// Extract the first complete token or S-expression from the start of `s`
fn extract_first_token_or_sexpr(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('(') {
        // Collect balanced S-expression
        let mut depth = 0usize;
        let mut result = String::new();
        for c in s.chars() {
            match c {
                '(' => {
                    depth += 1;
                    result.push(c);
                }
                ')' => {
                    depth -= 1;
                    result.push(c);
                    if depth == 0 {
                        return result;
                    }
                }
                _ => result.push(c),
            }
        }
        result
    } else {
        // Plain token (keyword, number, …)
        s.split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches(')')
            .to_string()
    }
}

// ───────────────────────────────────────────────────────
// .ktest Parser (concrete witness values)
// ───────────────────────────────────────────────────────

fn parse_ktest_file(path: &Path) -> Result<Vec<(String, i64)>> {
    let output = Command::new("ktest-tool").arg(path).output();

    let output = match output {
        Ok(o) => o,
        Err(_) => {
            // ktest-tool not available; try to parse binary directly
            return parse_ktest_binary(path);
        }
    };

    if !output.status.success() {
        return parse_ktest_binary(path);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut values = Vec::new();
    let mut current_name = String::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("name:") {
            // name: 'x'
            if let Some(start) = line.find('\'') {
                if let Some(end) = line[start + 1..].find('\'') {
                    current_name = line[start + 1..start + 1 + end].to_string();
                }
            }
        } else if (line.contains("int :") || line.contains("i32 :"))
            && !current_name.is_empty()
        {
            // data: 5
            // int : 5
            if let Some(colon_pos) = line.rfind(':') {
                let value_str = line[colon_pos + 1..].trim();
                // ktest-tool sometimes outputs hex
                let parsed = if value_str.starts_with("0x") {
                    i64::from_str_radix(&value_str[2..], 16).ok()
                } else {
                    value_str.parse::<i32>().ok().map(|v| v as i64)
                };
                if let Some(v) = parsed {
                    values.push((current_name.clone(), v));
                    current_name.clear();
                }
            }
        } else if line.starts_with("data:") && !current_name.is_empty() {
            // data: \x05\x00\x00\x00  (little-endian i32)
            // Try to read the bytes
            let data_part = line[5..].trim();
            if let Some(v) = parse_ktest_data_bytes(data_part) {
                values.push((current_name.clone(), v));
                current_name.clear();
            }
        }
    }

    Ok(values)
}

/// Very simple binary .ktest reader for when ktest-tool is unavailable.
/// Format: magic "KTEST" version(u32) num_args args... num_objects objects...
/// object: name_len(u32) name(bytes) data_len(u32) data(bytes)
fn parse_ktest_binary(path: &Path) -> Result<Vec<(String, i64)>> {
    let bytes = fs::read(path)?;
    let mut values = Vec::new();

    // Check magic
    if bytes.len() < 5 || &bytes[0..5] != b"KTEST" {
        return Ok(values);
    }

    let mut pos = 5;
    // version (4 bytes)
    if pos + 4 > bytes.len() {
        return Ok(values);
    }
    pos += 4;

    // num_args (4 bytes)
    if pos + 4 > bytes.len() {
        return Ok(values);
    }
    let num_args = u32::from_be_bytes([bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]]) as usize;
    pos += 4;

    // skip args
    for _ in 0..num_args {
        if pos + 4 > bytes.len() { return Ok(values); }
        let len = u32::from_be_bytes([bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]]) as usize;
        pos += 4 + len;
    }

    // num_objects
    if pos + 4 > bytes.len() { return Ok(values); }
    let num_objects = u32::from_be_bytes([bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]]) as usize;
    pos += 4;

    for _ in 0..num_objects {
        // name
        if pos + 4 > bytes.len() { break; }
        let name_len = u32::from_be_bytes([bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]]) as usize;
        pos += 4;
        if pos + name_len > bytes.len() { break; }
        let name = String::from_utf8_lossy(&bytes[pos..pos+name_len])
            .trim_end_matches('\0')
            .to_string();
        pos += name_len;

        // data
        if pos + 4 > bytes.len() { break; }
        let data_len = u32::from_be_bytes([bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]]) as usize;
        pos += 4;
        if pos + data_len > bytes.len() { break; }
        let data = &bytes[pos..pos+data_len];
        pos += data_len;

        // Interpret as little-endian i32 if 4 bytes
        if data_len == 4 {
            let val = i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as i64;
            values.push((name, val));
        }
    }

    Ok(values)
}

/// Parse ktest-tool "data:" line bytes like `\x05\x00\x00\x00`
fn parse_ktest_data_bytes(s: &str) -> Option<i64> {
    // Collect up to 4 bytes from \xHH escapes
    let mut bytes = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();
    while i < chars.len() && bytes.len() < 4 {
        if chars[i] == '\\' && i + 3 < chars.len() && chars[i+1] == 'x' {
            let hex: String = chars[i+2..i+4].iter().collect();
            if let Ok(b) = u8::from_str_radix(&hex, 16) {
                bytes.push(b);
                i += 4;
                continue;
            }
        }
        i += 1;
    }
    if bytes.len() == 4 {
        Some(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i64)
    } else {
        None
    }
}

// ───────────────────────────────────────────────────────
// Utility
// ───────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn display_klee_stats(klee_dir: &str) -> Result<()> {
    let stats_path = format!("{}/run.stats", klee_dir);
    if Path::new(&stats_path).exists() {
        let content = fs::read_to_string(&stats_path)?;
        println!("\nKLEE Statistics:\n{}", content);
    }
    Ok(())
}