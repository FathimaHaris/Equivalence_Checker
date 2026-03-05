// src/symbolic/mod.rs
// ═══════════════════════════════════════════════════════
// Module 5: Symbolic Execution using KLEE
//
// DFD Structure (matches diagram):
//   Instrumented IR
//     → 0.5.1  Symbolic Input Generation   → symbolic inputs
//     → 0.5.2  Path Exploration            → feasible execution paths
//     → 0.5.3  Constraint & Observable Extraction → path constraints & symbolic observables
//     → 0.5.4  Path Summary Construction   → symbolic path summaries
// ═══════════════════════════════════════════════════════

use crate::types::{
    AnalysisConfig, PathSummary, ProgramKind, CheckerError, ObservableEffects,
};
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SymbolicSummaries {
    pub c_summaries:    Vec<PathSummary>,
    pub rust_summaries: Vec<PathSummary>,
}

// ── Top-level entry point ─────────────────────────────
// Orchestrates all four DFD stages for both C and Rust IR.

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

// ── Full pipeline for one program ────────────────────
// Runs stages 0.5.1 → 0.5.2 → 0.5.3 → 0.5.4 in sequence.

fn run_symbolic_pipeline(
    ir_path:       &str,
    function_name: &str,
    max_paths:     u32,
    timeout:       u32,
    program_kind:  ProgramKind,
) -> Result<Vec<PathSummary>> {
    // ── 0.5.1  Symbolic Input Generation ─────────────
    let klee_out_dir = stage_051_symbolic_input_generation(
        ir_path, function_name, &program_kind,
    )?;

    // ── 0.5.2  Path Exploration ───────────────────────
    let test_numbers = stage_052_path_exploration(
        ir_path, function_name, max_paths, timeout, &program_kind, &klee_out_dir,
    )?;

    // ── 0.5.3  Constraint & Observable Extraction ────
    let raw_paths = stage_053_constraint_observable_extraction(
        &klee_out_dir, &test_numbers,
    )?;

    // ── 0.5.4  Path Summary Construction ─────────────
    let summaries = stage_054_path_summary_construction(raw_paths, &program_kind);

    Ok(summaries)
}

// ═══════════════════════════════════════════════════════
// 0.5.1  Symbolic Input Generation
// ═══════════════════════════════════════════════════════
// Input:  Instrumented IR (ir_path)
// Output: KLEE output directory path (symbolic inputs ready)

fn stage_051_symbolic_input_generation(
    _ir_path:      &str,
    function_name: &str,
    program_kind:  &ProgramKind,
) -> Result<String> {
    let kind_str = match program_kind { ProgramKind::C => "C", ProgramKind::Rust => "Rust" };
    let klee_out_dir = format!(
        "/tmp/equivalence_checker/klee_{}_{}",
        function_name, kind_str
    );

    // Clean up any previous run so KLEE starts fresh.
    if Path::new(&klee_out_dir).exists() {
        let _ = fs::remove_dir_all(&klee_out_dir);
    }

    println!("    [0.5.1] Symbolic input generation → {}", klee_out_dir);
    Ok(klee_out_dir)
}

// ═══════════════════════════════════════════════════════
// 0.5.2  Path Exploration
// ═══════════════════════════════════════════════════════
// Input:  Symbolic inputs (ir_path + klee_out_dir)
// Output: Feasible execution paths (test case numbers produced by KLEE)

fn stage_052_path_exploration(
    ir_path:       &str,
    function_name: &str,
    _max_paths:    u32,
    timeout:       u32,
    program_kind:  &ProgramKind,
    klee_out_dir:  &str,
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
       .arg("--max-memory=1000");

    match program_kind {
        ProgramKind::C    => cmd.arg("--entry-point=main"),
        ProgramKind::Rust => cmd.arg("--entry-point=klee_harness"),
    };
    cmd.arg(ir_path);

    let output = cmd.output()?;

    for line in String::from_utf8_lossy(&output.stderr).lines() {
        if line.contains("ERROR") || line.contains("WARNING") || line.contains("KLEE:") {
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

    // Collect the test case numbers that represent feasible execution paths.
    let mut test_numbers = Vec::new();
    for entry in fs::read_dir(klee_out_dir)? {
        let entry = entry?;
        let name  = entry.file_name();
        let s     = name.to_string_lossy();
        if s.starts_with("test") && s.ends_with(".ktest") {
            if let Ok(n) = s["test".len()..s.len()-".ktest".len()].parse::<usize>() {
                test_numbers.push(n);
            }
        }
    }
    test_numbers.sort();
    println!("    [0.5.2] Found {} feasible execution paths", test_numbers.len());
    Ok(test_numbers)
}

// ═══════════════════════════════════════════════════════
// 0.5.3  Constraint & Observable Extraction
// ═══════════════════════════════════════════════════════
// Input:  Feasible execution paths (test numbers + klee_out_dir)
// Output: Path constraints & symbolic observables (RawPathData per test)

struct RawPathData {
    test_num:    usize,
    constraints: Vec<String>,
    witness:     Vec<(String, i64)>,
}

fn stage_053_constraint_observable_extraction(
    klee_dir:     &str,
    test_numbers: &[usize],
) -> Result<Vec<RawPathData>> {
    println!("    [0.5.3] Extracting path constraints & symbolic observables…");

    let mut raw_paths = Vec::new();

    for &num in test_numbers {
        let kquery_path = format!("{}/test{:06}.kquery", klee_dir, num);
        let ktest_path  = format!("{}/test{:06}.ktest",  klee_dir, num);

        // Extract path constraints from the .kquery file.
        let constraints = if Path::new(&kquery_path).exists() {
            parse_kquery_constraints(Path::new(&kquery_path))?
        } else {
            vec!["true".to_string()]
        };

        // Extract symbolic observables (witness values) from the .ktest file.
        let witness = if Path::new(&ktest_path).exists() {
            let w = parse_ktest_binary(Path::new(&ktest_path))?;
            if w.is_empty() {
                println!("      [0.5.3] binary parse empty for test {:06}, trying ktest-tool…", num);
                parse_ktest_via_tool(Path::new(&ktest_path)).unwrap_or_default()
            } else { w }
        } else {
            vec![]
        };

        println!(
            "      test {:06}: {} constraints, witness={:?}",
            num, constraints.len(), witness
        );

        raw_paths.push(RawPathData { test_num: num, constraints, witness });
    }

    Ok(raw_paths)
}

// ═══════════════════════════════════════════════════════
// 0.5.4  Path Summary Construction
// ═══════════════════════════════════════════════════════
// Input:  Path constraints & symbolic observables (RawPathData)
// Output: Symbolic path summaries (Vec<PathSummary>)

fn stage_054_path_summary_construction(
    raw_paths:    Vec<RawPathData>,
    program_kind: &ProgramKind,
) -> Vec<PathSummary> {
    println!("    [0.5.4] Constructing path summaries…");

    let mut summaries: Vec<PathSummary> = raw_paths
        .into_iter()
        .map(|raw| PathSummary {
            id:          format!("{:?}-{}", program_kind, raw.test_num),
            program:     program_kind.clone(),
            constraints: raw.constraints,
            // kquery always ends with "false)" — no real return expression stored there.
            // The equivalence checker obtains return values by running the compiled binaries.
            return_expr: None,
            witness:     raw.witness,
            observables: ObservableEffects::default(),
        })
        .collect();

    // Guarantee at least one summary so downstream stages always have input.
    if summaries.is_empty() {
        summaries.push(PathSummary {
            id:          format!("{:?}-placeholder", program_kind),
            program:     program_kind.clone(),
            constraints: vec!["true".to_string()],
            return_expr: None,
            witness:     vec![],
            observables: ObservableEffects::default(),
        });
    }

    println!("    [0.5.4] {} symbolic path summaries constructed", summaries.len());
    summaries
}

// ── .kquery constraint parser ─────────────────────────

fn parse_kquery_constraints(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    let mut constraints = Vec::new();

    if let Some(start) = content.find("(query [") {
        let after = &content[start + "(query [".len()..];
        if let Some(end) = find_matching_square_bracket(after) {
            let section = &after[..end];
            let mut depth = 0usize;
            let mut cur   = String::new();
            for c in section.chars() {
                match c {
                    '(' => { if depth == 0 { cur.clear(); } depth += 1; cur.push(c); }
                    ')' if depth > 0 => {
                        depth -= 1; cur.push(c);
                        if depth == 0 {
                            let t = cur.trim().to_string();
                            if !t.is_empty() { constraints.push(t); }
                            cur.clear();
                        }
                    }
                    _ if depth > 0 => cur.push(c),
                    _ => {}
                }
            }
        }
    }

    if constraints.is_empty() { constraints.push("true".to_string()); }
    Ok(constraints)
}

fn find_matching_square_bracket(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ']' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

// ── .ktest binary parser (little-endian) ─────────────

/// KLEE .ktest format — ALL integers are LITTLE-ENDIAN.
/// magic(5) version(u32le) num_args(u32le) [arg_len(u32le) arg_bytes...]*
/// num_objects(u32le) [name_len(u32le) name_bytes data_len(u32le) data_bytes...]*
fn parse_ktest_binary(path: &Path) -> Result<Vec<(String, i64)>> {
    let bytes = fs::read(path)?;
    let mut vals = Vec::new();

    if bytes.len() < 5 || &bytes[0..5] != b"KTEST" {
        println!("      [ktest] bad magic in {:?}", path);
        return Ok(vals);
    }

    let mut pos = 5usize;

    macro_rules! read_u32le {
        () => {{
            if pos + 4 > bytes.len() { return Ok(vals); }
            let v = u32::from_le_bytes([bytes[pos], bytes[pos+1], bytes[pos+2], bytes[pos+3]]) as usize;
            pos += 4;
            v
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
    println!("      [ktest] {} objects in {:?}", num_objects, path.file_name().unwrap_or_default());

    for _ in 0..num_objects {
        let name_len = read_u32le!();
        if pos + name_len > bytes.len() { break; }
        let name = String::from_utf8_lossy(&bytes[pos..pos+name_len])
            .trim_end_matches('\0').to_string();
        pos += name_len;

        let data_len = read_u32le!();
        if pos + data_len > bytes.len() { break; }
        let data = &bytes[pos..pos+data_len];
        pos += data_len;

        let val: Option<i64> = match data_len {
            1 => Some(i64::from(data[0] as i8)),
            2 => Some(i16::from_le_bytes([data[0], data[1]]) as i64),
            4 => Some(i32::from_le_bytes([data[0], data[1], data[2], data[3]]) as i64),
            8 => Some(i64::from_le_bytes([
                data[0], data[1], data[2], data[3],
                data[4], data[5], data[6], data[7],
            ])),
            _ => None,
        };
        println!("      [ktest] name={:?} data_len={} val={:?}", name, data_len, val);
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

    let mut tool_path = None;
    for p in &ktest_tool_paths {
        if Path::new(p).exists() || p == &"ktest-tool" {
            tool_path = Some(*p);
            break;
        }
    }

    let tool = tool_path.unwrap_or("ktest-tool");
    let out = Command::new(tool).arg(path).output();

    let out = match out {
        Ok(o)  => o,
        Err(e) => {
            println!("      [ktest-tool] not available: {}", e);
            return Ok(vec![]);
        }
    };

    let text   = String::from_utf8_lossy(&out.stdout);
    let mut vals: Vec<(String, i64)> = Vec::new();
    let mut current_name: Option<String> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.contains("name:") {
            if let Some(start) = line.find('\'') {
                if let Some(end) = line[start+1..].find('\'') {
                    current_name = Some(line[start+1..start+1+end].to_string());
                }
            }
        }
        if line.contains(" int :") || line.contains(" int:") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if let Some(val_str) = parts.last() {
                let clean = val_str.replace(':', "").trim().to_string();
                if let Ok(v) = clean.parse::<i64>() {
                    if let Some(name) = current_name.take() {
                        vals.push((name, v));
                    }
                }
            }
        }
        if line.contains("data:") && line.contains("\\x") {
            let hex_part = line[line.find("\\x").unwrap()..].to_string();
            let hex_bytes: Vec<u8> = hex_part.split("\\x")
                .filter(|s| s.len() >= 2)
                .filter_map(|s| u8::from_str_radix(&s[..2], 16).ok())
                .collect();
            let val: Option<i64> = match hex_bytes.len() {
                1 => Some(hex_bytes[0] as i8 as i64),
                2 => Some(i16::from_le_bytes([hex_bytes[0], hex_bytes[1]]) as i64),
                4 => Some(i32::from_le_bytes([
                    hex_bytes[0], hex_bytes[1], hex_bytes[2], hex_bytes[3]
                ]) as i64),
                8 => Some(i64::from_le_bytes([
                    hex_bytes[0], hex_bytes[1], hex_bytes[2], hex_bytes[3],
                    hex_bytes[4], hex_bytes[5], hex_bytes[6], hex_bytes[7],
                ])),
                _ => None,
            };
            if let (Some(name), Some(v)) = (current_name.take(), val) {
                vals.push((name, v));
            }
        }
    }

    println!("      [ktest-tool] parsed witness: {:?}", vals);
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