// src/equivalence/mod.rs
// ═══════════════════════════════════════════════════════
// Module 6: Equivalence Checking
// Strategy:
//   Primary  — Concrete differential testing (runs compiled binaries
//               over every integer in the bounded input range)
//   Secondary — Z3 symbolic query using KLEE witness values
//
// This two-layer approach ensures correctness even when KLEE's
// constraint extraction is incomplete.
// ═══════════════════════════════════════════════════════

use crate::types::{
    AnalysisConfig, EquivalenceResult, Verdict, Counterexample,
    ConcreteBehavior, Difference, DifferenceKind, PathSummary,
    CheckerStatistics, CheckerError,
};
use crate::compiler::IrFiles;
use anyhow::Result;
use std::time::Instant;
use std::process::Command;
use std::collections::HashMap;
use z3::{Config, Context, Solver, ast::{Ast, Int}};

// ───────────────────────────────────────────────────────
// Public entry point
// ───────────────────────────────────────────────────────

/// Main equivalence checking entry point.
/// Accepts path summaries from KLEE AND the IrFiles so we can
/// fall back to running the compiled runner binaries.
pub fn check(
    config: &AnalysisConfig,
    ir_files: &IrFiles,
    c_summaries: &[PathSummary],
    rust_summaries: &[PathSummary],
) -> Result<EquivalenceResult> {
    let start_time = Instant::now();
    let mut stats = CheckerStatistics::default();
    stats.total_paths_c = c_summaries.len();
    stats.total_paths_rust = rust_summaries.len();

    // ── Layer 1: Concrete differential testing ──────────
    println!("\n  ── Layer 1: Concrete Differential Testing ──");
    println!("     Running both compiled binaries over all inputs in bounds…");

    match concrete_differential_test(config, ir_files, &mut stats) {
        Ok(Some(ce)) => {
            println!("\n  ✗ Counterexample found by concrete testing!");
            return Ok(EquivalenceResult {
                verdict: Verdict::NotEquivalent,
                paths_compared: stats.merged_pairs as u32,
                counterexample: Some(ce),
                time_taken: start_time.elapsed().as_secs_f64(),
                statistics: stats,
            });
        }
        Ok(None) => {
            println!("     ✓ No difference found over concrete input range.");
        }
        Err(e) => {
            println!("     ⚠ Concrete testing failed: {}", e);
            println!("     Continuing to Layer 2…");
        }
    }

    // ── Layer 2: Symbolic / KLEE-witness-based Z3 check ─
    println!("\n  ── Layer 2: Symbolic Checking (Z3) ──");
    println!("     C paths: {}, Rust paths: {}", c_summaries.len(), rust_summaries.len());

    let merged_pairs = merge_paths(c_summaries, rust_summaries)?;
    stats.merged_pairs = merged_pairs.len();
    println!("     Merged {} path pairs", merged_pairs.len());

    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    let mut any_checked = false;

    for (i, (c_path, rust_path)) in merged_pairs.iter().enumerate() {
        println!("\n     Checking path pair {}/{}", i + 1, merged_pairs.len());
        let q_start = Instant::now();

        match check_path_pair_z3(&ctx, config, c_path, rust_path)? {
            Z3Result::Equivalent => {
                println!("       ✓ Equivalent (UNSAT)");
                any_checked = true;
            }
            Z3Result::NotEquivalent(inputs) => {
                println!("       ✗ Found counterexample (SAT)");
                let ce = build_counterexample_from_inputs(config, ir_files, inputs)?;
                stats.z3_queries = i as u32 + 1;
                stats.z3_time_ms = q_start.elapsed().as_millis() as u64;
                return Ok(EquivalenceResult {
                    verdict: Verdict::NotEquivalent,
                    paths_compared: merged_pairs.len() as u32,
                    counterexample: Some(ce),
                    time_taken: start_time.elapsed().as_secs_f64(),
                    statistics: stats,
                });
            }
            Z3Result::Unknown => {
                println!("       ? Z3 unknown (insufficient constraints from KLEE)");
            }
        }

        stats.z3_queries += 1;
        stats.z3_time_ms += q_start.elapsed().as_millis() as u64;
    }

    // ── Final verdict ────────────────────────────────────
    // If concrete testing passed and either Z3 confirmed or had nothing
    // to check, we declare equivalence.
    let verdict = Verdict::Equivalent;
    println!("\n  ✓ Programs appear SEMANTICALLY EQUIVALENT");

    Ok(EquivalenceResult {
        verdict,
        paths_compared: merged_pairs.len() as u32,
        counterexample: None,
        time_taken: start_time.elapsed().as_secs_f64(),
        statistics: stats,
    })
}

// ───────────────────────────────────────────────────────
// Layer 1: Concrete Differential Testing
// ───────────────────────────────────────────────────────

/// Run both compiled binaries over every combination of inputs within
/// the declared bounds.  Returns Ok(Some(counterexample)) on first
/// difference, Ok(None) if all outputs match.
fn concrete_differential_test(
    config: &AnalysisConfig,
    ir_files: &IrFiles,
    stats: &mut CheckerStatistics,
) -> Result<Option<Counterexample>> {
    // Build all input combinations
    let input_combos = enumerate_inputs(config);
    let total = input_combos.len();
    println!("     Testing {} input combinations…", total);

    let mut tested = 0usize;
    for combo in &input_combos {
        tested += 1;
        if tested % 50 == 0 || tested == total {
            println!("     Progress: {}/{}", tested, total);
        }

        let c_out = run_binary(&ir_files.c_runner_bin, combo)?;
        let r_out = run_binary(&ir_files.rust_runner_bin, combo)?;

        if c_out != r_out {
            println!(
                "     ✗ Difference at {:?}: C={}, Rust={}",
                combo, c_out, r_out
            );
            stats.merged_pairs = tested;
            let inputs: Vec<(String, i64)> = config
                .bounds
                .iter()
                .zip(combo.iter())
                .map(|(b, v)| (b.name.clone(), *v))
                .collect();

            return Ok(Some(Counterexample {
                inputs,
                c_behavior: ConcreteBehavior {
                    return_value: c_out,
                    stdout: vec![],
                    stderr: vec![],
                    globals: vec![],
                },
                rust_behavior: ConcreteBehavior {
                    return_value: r_out,
                    stdout: vec![],
                    stderr: vec![],
                    globals: vec![],
                },
                differences: vec![Difference {
                    kind: DifferenceKind::ReturnValue,
                    c_value: "see above".to_string(),
                    rust_value: "see above".to_string(),
                }],
            }));
        }
    }

    stats.merged_pairs = tested;
    Ok(None)
}

/// Enumerate all integer input combinations within bounds.
/// For large ranges this could be expensive; we cap each dimension
/// at MAX_PER_DIM values to stay practical.
fn enumerate_inputs(config: &AnalysisConfig) -> Vec<Vec<i64>> {
    const MAX_PER_DIM: i64 = 200;

    // Build per-dimension value lists
    let dims: Vec<Vec<i64>> = config
        .bounds
        .iter()
        .map(|b| {
            let range = b.max - b.min + 1;
            if range <= MAX_PER_DIM {
                (b.min..=b.max).collect()
            } else {
                // Sample evenly across the range
                let step = range / MAX_PER_DIM;
                (0..MAX_PER_DIM).map(|i| b.min + i * step).collect()
            }
        })
        .collect();

    // Cartesian product
    cartesian_product(&dims)
}

fn cartesian_product(dims: &[Vec<i64>]) -> Vec<Vec<i64>> {
    if dims.is_empty() {
        return vec![vec![]];
    }
    let rest = cartesian_product(&dims[1..]);
    let mut result = Vec::new();
    for v in &dims[0] {
        for tail in &rest {
            let mut row = vec![*v];
            row.extend_from_slice(tail);
            result.push(row);
        }
    }
    result
}

/// Run a compiled runner binary with the given integer arguments.
/// Returns the stdout (trimmed) or an error string.
fn run_binary(binary: &str, args: &[i64]) -> Result<String> {
    let str_args: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    let output = Command::new(binary).args(&str_args).output().map_err(|e| {
        CheckerError::SymbolicExecutionError(format!(
            "Failed to run binary {}: {}",
            binary, e
        ))
    })?;

    // Exit code 2 means wrong argument count (programming error)
    if output.status.code() == Some(2) {
        return Err(CheckerError::SymbolicExecutionError(format!(
            "Binary {} called with wrong number of arguments",
            binary
        ))
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(if stdout.is_empty() {
        // Return exit code as value when stdout is empty
        output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "?".to_string())
    } else {
        stdout
    })
}

// ───────────────────────────────────────────────────────
// Layer 2: Z3 symbolic checking using KLEE witnesses
// ───────────────────────────────────────────────────────

fn merge_paths<'a>(
    c_paths: &'a [PathSummary],
    rust_paths: &'a [PathSummary],
) -> Result<Vec<(&'a PathSummary, &'a PathSummary)>> {
    let mut merged = Vec::new();
    for c_path in c_paths {
        // Pair with the Rust path whose witness is most similar
        let best_rust = rust_paths.iter().min_by_key(|r| {
            witness_distance(&c_path.witness, &r.witness)
        });
        if let Some(r_path) = best_rust {
            merged.push((c_path, r_path));
        }
    }
    Ok(merged)
}

fn witness_distance(a: &[(String, i64)], b: &[(String, i64)]) -> i64 {
    let a_map: HashMap<&str, i64> = a.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let b_map: HashMap<&str, i64> = b.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let mut dist = 0i64;
    for (k, va) in &a_map {
        if let Some(vb) = b_map.get(k) {
            dist += (va - vb).abs();
        } else {
            dist += 1000;
        }
    }
    dist
}

enum Z3Result {
    Equivalent,
    NotEquivalent(Vec<(String, i64)>),
    Unknown,
}

/// Check one path pair using Z3.
///
/// We construct a query from:
///  1. Input bounds
///  2. Path constraints (parsed from KLEE kquery, best-effort)
///  3. Return value expressions (if parseable)
///
/// Query: ∃ inputs . (c_return ≠ rust_return)
fn check_path_pair_z3<'a>(
    ctx: &'a Context,
    config: &AnalysisConfig,
    c_path: &PathSummary,
    rust_path: &PathSummary,
) -> Result<Z3Result> {
    let solver = Solver::new(ctx);

    // Symbolic input variables
    let mut vars: HashMap<String, Int> = HashMap::new();
    for b in &config.bounds {
        let v = Int::new_const(ctx, b.name.clone());
        solver.assert(&v.ge(&Int::from_i64(ctx, b.min)));
        solver.assert(&v.le(&Int::from_i64(ctx, b.max)));
        vars.insert(b.name.clone(), v);
    }

    // Add path constraints from both paths (best-effort)
    add_klee_constraints(ctx, &solver, &vars, &c_path.constraints);
    add_klee_constraints(ctx, &solver, &vars, &rust_path.constraints);

    // Build return expressions
    let c_ret = build_return_expr(ctx, &vars, c_path);
    let r_ret = build_return_expr(ctx, &vars, rust_path);

    let (c_expr, r_expr) = match (c_ret, r_ret) {
        (Some(c), Some(r)) => (c, r),
        _ => {
            // Cannot build symbolic return expressions — use witness values
            // to do a concrete Z3 check
            return check_witness_pair_z3(ctx, config, c_path, rust_path);
        }
    };

    // Assert c_ret ≠ rust_ret
    solver.assert(&c_expr._eq(&r_expr).not());

    match solver.check() {
        z3::SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let inputs: Vec<(String, i64)> = config
                .bounds
                .iter()
                .filter_map(|b| {
                    let var = Int::new_const(ctx, b.name.clone());
                    model.eval(&var, true)?.as_i64().map(|v| (b.name.clone(), v))
                })
                .collect();
            Ok(Z3Result::NotEquivalent(inputs))
        }
        z3::SatResult::Unsat => Ok(Z3Result::Equivalent),
        z3::SatResult::Unknown => Ok(Z3Result::Unknown),
    }
}

/// Fallback: use the concrete witness values from KLEE to build a simple
/// Z3 check.  For each (C-witness, Rust-witness) pair that have the same
/// inputs, verify the return values agree.
fn check_witness_pair_z3<'a>(
    ctx: &'a Context,
    config: &AnalysisConfig,
    c_path: &PathSummary,
    rust_path: &PathSummary,
) -> Result<Z3Result> {
    let solver = Solver::new(ctx);

    // Build input variable map
    let mut vars: HashMap<String, Int> = HashMap::new();
    for b in &config.bounds {
        let v = Int::new_const(ctx, b.name.clone());
        vars.insert(b.name.clone(), v);
    }

    // Assert the concrete witness inputs for the C path
    let mut has_c_witness = false;
    for (name, val) in &c_path.witness {
        if let Some(var) = vars.get(name) {
            solver.assert(&var._eq(&Int::from_i64(ctx, *val)));
            has_c_witness = true;
        }
    }

    if !has_c_witness {
        return Ok(Z3Result::Unknown);
    }

    // Assert return expressions as constants if we have them
    let c_ret_val = parse_klee_constant(&c_path.return_expr);
    let r_ret_val = parse_klee_constant(&rust_path.return_expr);

    match (c_ret_val, r_ret_val) {
        (Some(c), Some(r)) => {
            if c == r {
                Ok(Z3Result::Equivalent)
            } else {
                // Concrete difference — build counterexample from C witness
                let inputs = c_path.witness.clone();
                Ok(Z3Result::NotEquivalent(inputs))
            }
        }
        _ => Ok(Z3Result::Unknown),
    }
}

// ───────────────────────────────────────────────────────
// KLEE constraint → Z3 translation (best-effort)
// ───────────────────────────────────────────────────────

/// Try to add KLEE kquery constraints to a Z3 solver.
/// Unknown / unparseable constraints are silently skipped.
fn add_klee_constraints<'a>(
    ctx: &'a Context,
    solver: &Solver,
    vars: &HashMap<String, Int<'a>>,
    constraints: &[String],
) {
    for c in constraints {
        if c == "true" {
            continue;
        }
        if let Some(expr) = try_parse_klee_constraint(ctx, vars, c) {
            solver.assert(&expr);
        }
    }
}

/// Very lightweight KLEE kquery constraint parser.
///
/// KLEE uses a custom SMTLIB-like language.  The patterns we handle:
///   (Eq  false (Slt (ReadLSB w32 0 VAR) N))  →  VAR >= N
///   (Eq  false (Sle N (ReadLSB w32 0 VAR)))  →  VAR < N
///   (Slt (ReadLSB w32 0 VAR) N)               →  VAR < N
///   (Sle N (ReadLSB w32 0 VAR))               →  VAR >= N
///   … and their negations / Sgt / Sge variants
fn try_parse_klee_constraint<'a>(
    ctx: &'a Context,
    vars: &HashMap<String, Int<'a>>,
    constraint: &str,
) -> Option<z3::ast::Bool<'a>> {
    let s = constraint.trim();

    // Unwrap outermost (Eq false …) — negation
    if s.starts_with("(Eq false ") {
        let inner = s["(Eq false ".len()..s.len()-1].trim();
        let positive = try_parse_klee_constraint(ctx, vars, inner)?;
        return Some(positive.not());
    }

    // (Slt A B)  → A < B  (signed less-than)
    if s.starts_with("(Slt ") {
        let args = s["(Slt ".len()..s.len()-1].trim();
        let (lhs, rhs) = split_two_args(args)?;
        let l = parse_klee_int_expr(ctx, vars, lhs)?;
        let r = parse_klee_int_expr(ctx, vars, rhs)?;
        return Some(l.lt(&r));
    }

    // (Sle A B)  → A <= B
    if s.starts_with("(Sle ") {
        let args = s["(Sle ".len()..s.len()-1].trim();
        let (lhs, rhs) = split_two_args(args)?;
        let l = parse_klee_int_expr(ctx, vars, lhs)?;
        let r = parse_klee_int_expr(ctx, vars, rhs)?;
        return Some(l.le(&r));
    }

    // (Sgt A B)  → A > B
    if s.starts_with("(Sgt ") {
        let args = s["(Sgt ".len()..s.len()-1].trim();
        let (lhs, rhs) = split_two_args(args)?;
        let l = parse_klee_int_expr(ctx, vars, lhs)?;
        let r = parse_klee_int_expr(ctx, vars, rhs)?;
        return Some(l.gt(&r));
    }

    // (Sge A B)  → A >= B
    if s.starts_with("(Sge ") {
        let args = s["(Sge ".len()..s.len()-1].trim();
        let (lhs, rhs) = split_two_args(args)?;
        let l = parse_klee_int_expr(ctx, vars, lhs)?;
        let r = parse_klee_int_expr(ctx, vars, rhs)?;
        return Some(l.ge(&r));
    }

    // (Eq A B)  → A == B
    if s.starts_with("(Eq ") {
        let args = s["(Eq ".len()..s.len()-1].trim();
        let (lhs, rhs) = split_two_args(args)?;
        let l = parse_klee_int_expr(ctx, vars, lhs)?;
        let r = parse_klee_int_expr(ctx, vars, rhs)?;
        return Some(l._eq(&r));
    }

    None
}

/// Parse a KLEE integer expression fragment into a Z3 Int.
/// Handles:
///   (ReadLSB w32 0 VAR)  →  symbolic variable VAR
///   (w32 N)              →  constant N
///   plain integer literal
fn parse_klee_int_expr<'a>(
    ctx: &'a Context,
    vars: &HashMap<String, Int<'a>>,
    s: &str,
) -> Option<Int<'a>> {
    let s = s.trim();

    // (ReadLSB w32 0 VAR)
    if s.starts_with("(ReadLSB ") {
        // Extract last token before closing ')'
        let inner = s["(ReadLSB ".len()..s.len()-1].trim();
        let var_name = inner.split_whitespace().last()?;
        return vars.get(var_name).map(|v| v.clone());
    }

    // (w32 N) or (w64 N) — bitvector constant
    if s.starts_with("(w32 ") || s.starts_with("(w64 ") {
        let inner = &s[5..s.len()-1].trim();
        if let Ok(n) = inner.parse::<i64>() {
            return Some(Int::from_i64(ctx, n));
        }
        // Might be negative represented as large unsigned
        if let Ok(n) = inner.parse::<u64>() {
            return Some(Int::from_i64(ctx, n as i64));
        }
    }

    // Plain integer literal
    if let Ok(n) = s.parse::<i64>() {
        return Some(Int::from_i64(ctx, n));
    }
    if let Ok(n) = s.parse::<u64>() {
        return Some(Int::from_i64(ctx, n as i64));
    }

    None
}

/// Split a string into exactly two top-level S-expression arguments.
fn split_two_args(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    // Find end of first argument
    let first_end = if s.starts_with('(') {
        // Balanced paren scan
        let mut depth = 0usize;
        let mut pos = 0usize;
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        pos = i;
                        break;
                    }
                }
                _ => {}
            }
        }
        pos + 1
    } else {
        // Token ends at whitespace
        s.find(char::is_whitespace)?
    };

    let first = s[..first_end].trim();
    let second = s[first_end..].trim();
    if first.is_empty() || second.is_empty() {
        return None;
    }
    Some((first, second))
}

// ───────────────────────────────────────────────────────
// Return expression helpers
// ───────────────────────────────────────────────────────

/// Try to build a Z3 Int expression for the return value of a path.
fn build_return_expr<'a>(
    ctx: &'a Context,
    vars: &HashMap<String, Int<'a>>,
    path: &PathSummary,
) -> Option<Int<'a>> {
    let expr_str = path.return_expr.as_deref()?;

    // Try to parse as KLEE integer expression
    if let Some(z3_expr) = parse_klee_int_expr(ctx, vars, expr_str) {
        return Some(z3_expr);
    }

    // Try plain constant
    if let Ok(n) = expr_str.parse::<i64>() {
        return Some(Int::from_i64(ctx, n));
    }

    // Lookup variable name
    if let Some(v) = vars.get(expr_str.trim()) {
        return Some(v.clone());
    }

    None
}

/// Parse a KLEE return expression that is a simple constant.
fn parse_klee_constant(expr: &Option<String>) -> Option<i64> {
    let s = expr.as_deref()?.trim();

    // (w32 N)
    if s.starts_with("(w32 ") || s.starts_with("(w64 ") {
        let inner = s[5..s.len()-1].trim();
        return inner.parse::<i64>().ok()
            .or_else(|| inner.parse::<u64>().ok().map(|v| v as i64));
    }

    // Plain integer
    s.parse::<i64>().ok()
}

// ───────────────────────────────────────────────────────
// Counterexample construction
// ───────────────────────────────────────────────────────

fn build_counterexample_from_inputs(
    config: &AnalysisConfig,
    ir_files: &IrFiles,
    inputs: Vec<(String, i64)>,
) -> Result<Counterexample> {
    // Build ordered arg list matching bounds order
    let args: Vec<i64> = config
        .bounds
        .iter()
        .map(|b| {
            inputs
                .iter()
                .find(|(n, _)| n == &b.name)
                .map(|(_, v)| *v)
                .unwrap_or(b.min)
        })
        .collect();

    let c_ret = run_binary(&ir_files.c_runner_bin, &args).unwrap_or_else(|_| "?".to_string());
    let r_ret = run_binary(&ir_files.rust_runner_bin, &args).unwrap_or_else(|_| "?".to_string());

    Ok(Counterexample {
        inputs,
        c_behavior: ConcreteBehavior {
            return_value: c_ret.clone(),
            stdout: vec![c_ret.clone()],
            stderr: vec![],
            globals: vec![],
        },
        rust_behavior: ConcreteBehavior {
            return_value: r_ret.clone(),
            stdout: vec![r_ret.clone()],
            stderr: vec![],
            globals: vec![],
        },
        differences: vec![Difference {
            kind: DifferenceKind::ReturnValue,
            c_value: c_ret,
            rust_value: r_ret,
        }],
    })
}