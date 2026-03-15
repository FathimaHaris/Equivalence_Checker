// src/symbolic/mod.rs
// ═══════════════════════════════════════════════════════
// Module 5: Symbolic Execution using KLEE
// ═══════════════════════════════════════════════════════

use crate::types::{
    AnalysisConfig, PathSummary, ProgramKind, CheckerError, ObservableEffects,
};
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SymbolicSummaries {
    pub c_summaries:    Vec<PathSummary>,
    pub rust_summaries: Vec<PathSummary>,
}

pub fn execute(
    config: &AnalysisConfig,
    files:  &crate::instrumentor::InstrumentedFiles,
) -> Result<SymbolicSummaries> {
    println!("  Running symbolic execution on C IR...");
    let c_summaries = run_symbolic_pipeline(
        &files.c_instrumented_path,
        &config.function_name,
        config.max_paths,
        config.timeout,
        ProgramKind::C,
    )?;
    println!("    → Found {} C paths", c_summaries.len());

    println!("  Running symbolic execution on Rust IR...");
    let rust_summaries = run_symbolic_pipeline(
        &files.rust_instrumented_path,
        &config.function_name,
        config.max_paths,
        config.timeout,
        ProgramKind::Rust,
    )?;
    println!("    → Found {} Rust paths", rust_summaries.len());

    if c_summaries.len() <= 1 || rust_summaries.len() <= 1 {
        println!("    ⚠ Warning: KLEE may have missed some paths.");
        println!("      C paths: {}, Rust paths: {}", c_summaries.len(), rust_summaries.len());
    }

    Ok(SymbolicSummaries { c_summaries, rust_summaries })
}

fn run_symbolic_pipeline(
    ir_path:       &str,
    function_name: &str,
    max_paths:     u32,
    timeout:       u32,
    program_kind:  ProgramKind,
) -> Result<Vec<PathSummary>> {
    let klee_out_dir = stage_051(ir_path, function_name, &program_kind)?;
    let test_numbers = stage_052(ir_path, function_name, max_paths, timeout, &program_kind, &klee_out_dir)?;
    let raw_paths    = stage_053(&klee_out_dir, &test_numbers)?;
    Ok(stage_054(raw_paths, &program_kind))
}

// ═══════════════════════════════════════════════════════
// 0.5.1
// ═══════════════════════════════════════════════════════
fn stage_051(_ir_path: &str, function_name: &str, program_kind: &ProgramKind) -> Result<String> {
    let kind_str = match program_kind { ProgramKind::C => "C", ProgramKind::Rust => "Rust" };
    let dir = format!("/tmp/equivalence_checker/klee_{}_{}", function_name, kind_str);
    if Path::new(&dir).exists() { let _ = fs::remove_dir_all(&dir); }
    println!("    [0.5.1] Symbolic input generation → {}", dir);
    Ok(dir)
}

// ═══════════════════════════════════════════════════════
// 0.5.2
// ═══════════════════════════════════════════════════════
fn stage_052(
    ir_path: &str, function_name: &str, _max_paths: u32, timeout: u32,
    program_kind: &ProgramKind, klee_out_dir: &str,
) -> Result<Vec<usize>> {
    println!("    [0.5.2] Path exploration (KLEE, up to {}s)…", timeout);

    let mut cmd = Command::new("/home/fathima/klee/build/bin/klee");
    cmd.arg("--output-dir").arg(klee_out_dir)
       .arg("--optimize=false")
       .arg("--search=dfs")
       .arg("--max-forks=500")
       .arg("--max-depth=500")
       .arg("--max-tests=200")
       .arg(format!("--max-time={}", timeout))
       .arg("--simplify-sym-indices")
       .arg("--write-test-info")
       .arg("--write-paths")
       .arg("--write-kqueries")
       .arg("--write-smt2s")
       .arg("--max-memory=1000");

    match program_kind {
        ProgramKind::C    => cmd.arg("--entry-point=main"),
        ProgramKind::Rust => cmd.arg("--entry-point=klee_harness"),
    };
    cmd.arg(ir_path);

    let output = cmd.output()?;
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        if (line.contains("ERROR") || line.contains("WARNING") || line.contains("KLEE:"))
   && !line.contains("provably false") {
            println!("    [KLEE] {}", line);
        }
    }

    if !Path::new(klee_out_dir).exists() {
        return Err(CheckerError::SymbolicExecutionError(format!(
            "KLEE failed for {} ({})\n{}",
            function_name,
            match program_kind { ProgramKind::C => "C", ProgramKind::Rust => "Rust" },
            String::from_utf8_lossy(&output.stderr)
        )).into());
    }

    let mut test_numbers = Vec::new();
    if let Ok(entries) = fs::read_dir(klee_out_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let s    = name.to_string_lossy();
            if s.starts_with("test") && s.ends_with(".ktest") {
                if let Ok(n) = s["test".len()..s.len() - ".ktest".len()].parse::<usize>() {
                    test_numbers.push(n);
                }
            }
        }
    }
    test_numbers.sort();
    println!("    [0.5.2] Found {} feasible execution paths", test_numbers.len());
    Ok(test_numbers)
}

// ═══════════════════════════════════════════════════════
// 0.5.3
// ═══════════════════════════════════════════════════════
struct RawPathData {
    test_num:    usize,
    constraints: Vec<String>,
    return_expr: Option<String>,
    witness:     Vec<(String, i64)>,
    label_map:   HashMap<String, String>,
}

fn stage_053(klee_dir: &str, test_numbers: &[usize]) -> Result<Vec<RawPathData>> {
    println!("    [0.5.3] Extracting path constraints & symbolic observables…");

    let mut raw_paths = Vec::new();

    for &num in test_numbers {
        let kquery_path = format!("{}/test{:06}.kquery", klee_dir, num);
        let ktest_path  = format!("{}/test{:06}.ktest",  klee_dir, num);

        let (constraints, return_expr, label_map) = if Path::new(&kquery_path).exists() {
            parse_kquery(Path::new(&kquery_path))?
        } else {
            (vec!["true".to_string()], None, HashMap::new())
        };

        let witness = if Path::new(&ktest_path).exists() {
            let w = parse_ktest_binary(Path::new(&ktest_path))?;
            if w.is_empty() {
                parse_ktest_via_tool(Path::new(&ktest_path)).unwrap_or_default()
            } else {
                w
            }
        } else {
            vec![]
        };

        println!(
            "      test {:06}: {} constraints, ret_expr={:?}, labels={:?}, witness={:?}",
            num,
            constraints.len(),
            return_expr.as_deref().map(|s| &s[..s.len().min(60)]),
            label_map,
            witness,
        );

        raw_paths.push(RawPathData { test_num: num, constraints, return_expr, witness, label_map });
    }

    Ok(raw_paths)
}

// ═══════════════════════════════════════════════════════
// 0.5.4
// ═══════════════════════════════════════════════════════
fn stage_054(raw_paths: Vec<RawPathData>, program_kind: &ProgramKind) -> Vec<PathSummary> {
    println!("    [0.5.4] Constructing path summaries…");

    let mut summaries: Vec<PathSummary> = raw_paths
        .into_iter()
        .map(|raw| PathSummary {
            id:          format!("{:?}-{}", program_kind, raw.test_num),
            program:     program_kind.clone(),
            constraints: raw.constraints,
            return_expr: raw.return_expr,
            witness:     raw.witness,
            observables: ObservableEffects::default(),
            label_map:   raw.label_map,
        })
        .collect();

    if summaries.is_empty() {
        summaries.push(PathSummary {
            id:          format!("{:?}-placeholder", program_kind),
            program:     program_kind.clone(),
            constraints: vec!["true".to_string()],
            return_expr: None,
            witness:     vec![],
            observables: ObservableEffects::default(),
            label_map:   HashMap::new(),
        });
    }

    println!("    [0.5.4] {} symbolic path summaries constructed", summaries.len());
    summaries
}

// ═══════════════════════════════════════════════════════
// .kquery parser
//
// KLEE .kquery format on this build:
//   array a[4] : w32 -> w8 = symbolic
//   array result[4] : w32 -> w8 = symbolic
//   (query [
//     (Sle 0 N0:(ReadLSB w32 0 a))
//     (Sle N0 100)
//     (Eq (ReadLSB w32 0 result) N0)   ← result binding
//   ] false)
//
// We extract:
//   constraints = all non-result constraints
//   return_expr = RHS of the result binding (e.g. "N0")
//   label_map   = {"N0": "a", "N1": "b", ...}
// ═══════════════════════════════════════════════════════

fn parse_kquery(path: &Path) -> Result<(Vec<String>, Option<String>, HashMap<String, String>)> {
    let content = fs::read_to_string(path)?;

    let query_start = match content.find("(query") {
        Some(p) => p,
        None    => return Ok((vec!["true".to_string()], None, HashMap::new())),
    };

    let body = &content[query_start..];
    let sections = collect_bracket_sections(body);

    let raw_constraints = if !sections.is_empty() {
        parse_constraint_section(&sections[0])
    } else {
        vec!["true".to_string()]
    };

    // Build label map: N0:(ReadLSB w32 0 a) → {"N0": "a"}
    let label_map = extract_label_map(&sections.get(0).map(|s| s.as_str()).unwrap_or(""));

    // Extract return expression from the result-binding constraint
    let return_expr = extract_result_from_constraints(&raw_constraints);

    // Filter out the result-binding constraint — it's output definition, not a path condition
    let constraints: Vec<String> = raw_constraints
        .into_iter()
        .filter(|c| !is_result_binding(c))
        .collect();

    Ok((constraints, return_expr, label_map))
}

/// Build a map from KLEE inline labels to variable names.
/// Scans for patterns like: N0:(ReadLSB w32 0 varname)
/// and produces {"N0" -> "varname"}.
fn extract_label_map(section: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut s = section;
    while let Some(colon_pos) = s.find(":(ReadLSB ") {
        // The label is the identifier immediately before the colon
        let before = &s[..colon_pos];
        let label: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<String>()
            .chars()
            .rev()
            .collect();

        let after = &s[colon_pos + 1..]; // starts at '('
        // Find matching ')' for the (ReadLSB ...) expression
        let mut depth = 0usize;
        let mut end = 0usize;
        for (i, c) in after.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 { end = i; break; }
                }
                _ => {}
            }
        }
        if end > 0 {
            let inner = after[1..end].trim(); // strip outer parens
            // inner = "w32 0 varname" — varname is last token
            if let Some(varname) = inner.split_whitespace().last() {
                if !label.is_empty() {
                    map.insert(label, varname.to_string());
                }
            }
            s = &after[end + 1..];
        } else {
            break;
        }
    }
    map
}

/// Find the constraint that binds the result variable and extract its RHS.
/// Looks for: (Eq (ReadLSB w32 0 result) EXPR) or (Eq EXPR (ReadLSB w32 0 result))
/// Returns EXPR as the return expression string.
fn extract_result_from_constraints(constraints: &[String]) -> Option<String> {
    for c in constraints {
        if c.contains("result") {
            if let Some(expr) = extract_result_rhs(c) {
                return Some(expr);
            }
        }
    }
    None
}

fn is_result_binding(c: &str) -> bool {
    c.contains("result") && (c.contains("ReadLSB") || c.contains("ReadMSB"))
}

fn extract_result_rhs(constraint: &str) -> Option<String> {
    let s = constraint.trim();
    if !s.starts_with("(Eq ") { return None; }

    // Strip "(Eq " prefix and the outer trailing ")"
    let inner = s["(Eq ".len()..].trim();
    let inner = if inner.ends_with(')') { &inner[..inner.len() - 1] } else { inner };
    let inner = inner.trim();

    let (first, rest) = split_balanced(inner)?;
    let rest = rest.trim();

    if first.contains("result") && (first.contains("ReadLSB") || first.contains("ReadMSB")) {
        // (Eq (ReadLSB result) RHS) → return RHS
        return Some(get_full_expr(rest));
    }
    if rest.contains("result") {
        // (Eq LHS (ReadLSB result)) → return LHS
        return Some(first.to_string());
    }
    None
}

/// Walk (query ...) and collect the inner text of every top-level [...] block.
fn collect_bracket_sections(body: &str) -> Vec<String> {
    let mut sections: Vec<String> = Vec::new();
    let mut paren_depth:   i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut section_start: Option<usize> = None;
    let bytes = body.as_bytes();
    let len   = bytes.len();
    let mut i = 0usize;

    while i < len {
        match bytes[i] {
            b'(' => { paren_depth += 1; }
            b')' => {
                paren_depth -= 1;
                if paren_depth < 0 { break; }
            }
            b'[' => {
                if bracket_depth == 0 {
                    section_start = Some(i + 1);
                }
                bracket_depth += 1;
            }
            b']' => {
                bracket_depth -= 1;
                if bracket_depth == 0 {
                    if let Some(start) = section_start.take() {
                        sections.push(body[start..i].to_string());
                    }
                }
            }
            b'"' => {
                i += 1;
                while i < len && bytes[i] != b'"' {
                    if bytes[i] == b'\\' { i += 1; }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    sections
}

fn parse_constraint_section(section: &str) -> Vec<String> {
    let mut constraints = Vec::new();
    let mut depth = 0usize;
    let mut cur   = String::new();

    for c in section.chars() {
        match c {
            '(' => {
                if depth == 0 { cur.clear(); }
                depth += 1;
                cur.push(c);
            }
            ')' if depth > 0 => {
                depth -= 1;
                cur.push(c);
                if depth == 0 {
                    let t = cur.trim().to_string();
                    if !t.is_empty() { constraints.push(t); }
                    cur.clear();
                }
            }
            _ if depth > 0 => { cur.push(c); }
            _ => {}
        }
    }

    if constraints.is_empty() { constraints.push("true".to_string()); }
    constraints
}

/// Extract the first complete balanced expression from s.
/// Returns (first_expr, remainder_after_first_expr).
fn split_balanced(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    if s.is_empty() { return None; }
    let end = if s.starts_with('(') {
        let mut depth = 0usize;
        let mut ep    = 0usize;
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => { depth -= 1; if depth == 0 { ep = i; break; } }
                _ => {}
            }
        }
        ep + 1
    } else {
        s.find(char::is_whitespace).unwrap_or(s.len())
    };
    let first  = s[..end].trim();
    let second = if end < s.len() { s[end..].trim() } else { "" };
    if first.is_empty() { None } else { Some((first, second)) }
}

/// Return the first complete SMT expression from s (handles nested parens).
fn get_full_expr(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() { return String::new(); }
    if s.starts_with('(') {
        let mut depth = 0usize;
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => { depth -= 1; if depth == 0 { return s[..=i].to_string(); } }
                _ => {}
            }
        }
        s.to_string()
    } else {
        s.split(char::is_whitespace).next().unwrap_or("").to_string()
    }
}

// ── .ktest binary parser ──────────────────────────────

fn parse_ktest_binary(path: &Path) -> Result<Vec<(String, i64)>> {
    let bytes = fs::read(path)?;
    let mut vals = Vec::new();

    if bytes.len() < 5 || &bytes[0..5] != b"KTEST" {
        return Ok(vals);
    }

    let mut pos = 5usize;
    macro_rules! read_u32le {
        () => {{
            if pos + 4 > bytes.len() { return Ok(vals); }
            let v = u32::from_le_bytes([bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]]);
            pos += 4;
            v as usize
        }};
    }

    let _version    = read_u32le!();
    let num_args    = read_u32le!();
    for _ in 0..num_args {
        let len = read_u32le!();
        if pos + len > bytes.len() { return Ok(vals); }
        pos += len;
    }

    let num_objects = read_u32le!();
    for _ in 0..num_objects {
        let name_len = read_u32le!();
        if pos + name_len > bytes.len() { break; }
        let name = String::from_utf8_lossy(&bytes[pos..pos + name_len])
            .trim_end_matches('\0')
            .to_string();
        pos += name_len;

        let data_len = read_u32le!();
        if pos + data_len > bytes.len() { break; }
        let data = &bytes[pos..pos + data_len];
        pos += data_len;

        let val: Option<i64> = match data_len {
            1 => Some(data[0] as i8 as i64),
            2 => Some(i16::from_le_bytes([data[0], data[1]]) as i64),
            4 => Some(i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as i64),
            8 => Some(i64::from_le_bytes([
                data[0], data[1], data[2], data[3],
                data[4], data[5], data[6], data[7],
            ])),
            _ => None,
        };
        if let Some(v) = val { vals.push((name, v)); }
    }
    Ok(vals)
}

// ── ktest-tool fallback ───────────────────────────────

fn parse_ktest_via_tool(path: &Path) -> Result<Vec<(String, i64)>> {
    let ktest_tool_paths = [
        "/home/fathima/klee/build/bin/ktest-tool",
        "/usr/local/bin/ktest-tool",
        "/usr/bin/ktest-tool",
        "ktest-tool",
    ];
    let tool = ktest_tool_paths
        .iter()
        .find(|p| Path::new(p).exists() || **p == "ktest-tool")
        .copied()
        .unwrap_or("ktest-tool");

    let out = match Command::new(tool).arg(path).output() {
        Ok(o)  => o,
        Err(e) => { println!("      [ktest-tool] not available: {}", e); return Ok(vec![]); }
    };

    let text = String::from_utf8_lossy(&out.stdout);
    let mut vals: Vec<(String, i64)> = Vec::new();
    let mut current_name: Option<String> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.contains("name:") {
            if let Some(start) = line.find('\'') {
                if let Some(end) = line[start + 1..].find('\'') {
                    current_name = Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
        if line.contains(" int :") || line.contains(" int:") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if let Some(val_str) = parts.last() {
                let clean = val_str.replace(':', "").trim().to_string();
                if let Ok(v) = clean.parse::<i64>() {
                    if let Some(name) = current_name.take() { vals.push((name, v)); }
                }
            }
        }
        if line.starts_with("data:") && line.contains("\\x") {
            if let Some(hex_pos) = line.find("\\x") {
                let hex_part = &line[hex_pos..];
                let hex_bytes: Vec<u8> = hex_part.split("\\x")
                    .filter(|s| s.len() >= 2)
                    .filter_map(|s| u8::from_str_radix(&s[..2], 16).ok())
                    .collect();
                let val: Option<i64> = match hex_bytes.len() {
                    1 => Some(hex_bytes[0] as i8 as i64),
                    2 => Some(i16::from_le_bytes([hex_bytes[0], hex_bytes[1]]) as i64),
                    4 => Some(i32::from_le_bytes([hex_bytes[0], hex_bytes[1], hex_bytes[2], hex_bytes[3]]) as i64),
                    8 => Some(i64::from_le_bytes([
                        hex_bytes[0], hex_bytes[1], hex_bytes[2], hex_bytes[3],
                        hex_bytes[4], hex_bytes[5], hex_bytes[6], hex_bytes[7],
                    ])),
                    _ => None,
                };
                if let (Some(name), Some(v)) = (current_name.take(), val) { vals.push((name, v)); }
            }
        }
    }
    Ok(vals)
}

#[allow(dead_code)]
pub fn display_klee_stats(klee_dir: &str) -> Result<()> {
    let info_path = format!("{}/info", klee_dir);
    if Path::new(&info_path).exists() {
        println!("\nKLEE Info:\n{}", fs::read_to_string(&info_path)?);
    }
    Ok(())
}