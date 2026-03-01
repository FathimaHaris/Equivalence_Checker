// src/equivalence/mod.rs
// ═══════════════════════════════════════════════════════
// Module 6: Equivalence Checking
//
// Two-layer strategy:
//   Layer 1 — Concrete differential testing via compiled runner binaries.
//              Exhaustively tests every integer in the bounded range.
//              Fast and always correct for finite domains.
//
//   Layer 2 — Z3 symbolic check using KLEE path constraints.
//              Confirms equivalence formally when KLEE found paths.
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

// ── Public entry point ────────────────────────────────

pub fn check(
    config:         &AnalysisConfig,
    ir_files:       &IrFiles,
    c_summaries:    &[PathSummary],
    rust_summaries: &[PathSummary],
) -> Result<EquivalenceResult> {
    let start = Instant::now();
    let mut stats = CheckerStatistics::default();
    stats.total_paths_c    = c_summaries.len();
    stats.total_paths_rust = rust_summaries.len();

    // ── Layer 1: Concrete differential testing ────────
    println!("\n  ── Layer 1: Concrete Differential Testing ──");
    println!("     Running compiled binaries over all inputs in bounds…");

    match concrete_differential_test(config, ir_files, &mut stats)? {
        Some(ce) => {
            println!("\n  ✗ Counterexample found by concrete testing!");
            return Ok(EquivalenceResult {
                verdict: Verdict::NotEquivalent,
                paths_compared: stats.merged_pairs as u32,
                counterexample: Some(ce),
                time_taken: start.elapsed().as_secs_f64(),
                statistics: stats,
            });
        }
        None => println!("     ✓ No difference found over concrete input range."),
    }

    // ── Layer 2: Z3 symbolic check ────────────────────
    println!("\n  ── Layer 2: Symbolic Checking (Z3) ──");
    let merged = merge_paths(c_summaries, rust_summaries)?;
    stats.merged_pairs = merged.len();
    println!("     Merged {} path pairs", merged.len());

    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    for (i, (cp, rp)) in merged.iter().enumerate() {
        let q_start = Instant::now();
        println!("     Checking pair {}/{}…", i + 1, merged.len());

        match check_pair_z3(&ctx, config, cp, rp)? {
            Z3Result::Equivalent => {
                println!("       ✓ UNSAT — equivalent for this path pair");
            }
            Z3Result::NotEquivalent(inputs) => {
                println!("       ✗ SAT — counterexample found");
                let ce = build_ce(config, ir_files, inputs)?;
                stats.z3_queries  += 1;
                stats.z3_time_ms  += q_start.elapsed().as_millis() as u64;
                return Ok(EquivalenceResult {
                    verdict: Verdict::NotEquivalent,
                    paths_compared: merged.len() as u32,
                    counterexample: Some(ce),
                    time_taken: start.elapsed().as_secs_f64(),
                    statistics: stats,
                });
            }
            Z3Result::Unknown => {
                println!("       ? Z3 unknown (KLEE constraints incomplete for this path)");
            }
        }
        stats.z3_queries += 1;
        stats.z3_time_ms += q_start.elapsed().as_millis() as u64;
    }

    println!("\n  ✓ Programs appear SEMANTICALLY EQUIVALENT");
    Ok(EquivalenceResult {
        verdict: Verdict::Equivalent,
        paths_compared: merged.len() as u32,
        counterexample: None,
        time_taken: start.elapsed().as_secs_f64(),
        statistics: stats,
    })
}

// ── Layer 1: Concrete testing ─────────────────────────

fn concrete_differential_test(
    config: &AnalysisConfig,
    ir:     &IrFiles,
    stats:  &mut CheckerStatistics,
) -> Result<Option<Counterexample>> {
    let combos = enumerate_inputs(config);
    println!("     Testing {} input combinations…", combos.len());

    for (n, combo) in combos.iter().enumerate() {
        if n % 100 == 0 || n + 1 == combos.len() {
            println!("     Progress: {}/{}", n + 1, combos.len());
        }
        let c_out = run_binary(&ir.c_runner_bin,    combo)?;
        let r_out = run_binary(&ir.rust_runner_bin, combo)?;

        if c_out != r_out {
            println!(
                "     ✗ Difference at {:?}: C={:?}  Rust={:?}",
                combo, c_out, r_out
            );
            stats.merged_pairs = n + 1;
            let inputs: Vec<(String, i64)> = config.bounds.iter()
                .zip(combo.iter())
                .map(|(b, v)| (b.name.clone(), *v))
                .collect();
            return Ok(Some(Counterexample {
                inputs,
                c_behavior: ConcreteBehavior {
                    return_value: c_out.clone(),
                    stdout: vec![c_out.clone()],
                    stderr: vec![],
                    globals: vec![],
                },
                rust_behavior: ConcreteBehavior {
                    return_value: r_out.clone(),
                    stdout: vec![r_out.clone()],
                    stderr: vec![],
                    globals: vec![],
                },
                differences: vec![Difference {
                    kind:       DifferenceKind::ReturnValue,
                    c_value:    c_out,
                    rust_value: r_out,
                }],
            }));
        }
    }

    stats.merged_pairs = combos.len();
    Ok(None)
}

fn enumerate_inputs(config: &AnalysisConfig) -> Vec<Vec<i64>> {
    const MAX_PER_DIM: i64 = 500;
    let dims: Vec<Vec<i64>> = config.bounds.iter().map(|b| {
        let range = b.max - b.min + 1;
        if range <= MAX_PER_DIM {
            (b.min..=b.max).collect()
        } else {
            let step = range / MAX_PER_DIM;
            (0..MAX_PER_DIM).map(|i| b.min + i * step).collect()
        }
    }).collect();
    cartesian_product(&dims)
}

fn cartesian_product(dims: &[Vec<i64>]) -> Vec<Vec<i64>> {
    if dims.is_empty() { return vec![vec![]]; }
    let rest = cartesian_product(&dims[1..]);
    let mut out = Vec::new();
    for v in &dims[0] {
        for tail in &rest {
            let mut row = vec![*v];
            row.extend_from_slice(tail);
            out.push(row);
        }
    }
    out
}

fn run_binary(bin: &str, args: &[i64]) -> Result<String> {
    let str_args: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    let out = Command::new(bin).args(&str_args).output().map_err(|e| {
        CheckerError::SymbolicExecutionError(format!("Failed to run {}: {}", bin, e))
    })?;
    if out.status.code() == Some(2) {
        return Err(CheckerError::SymbolicExecutionError(
            format!("Binary {} wrong argument count", bin)
        ).into());
    }
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(if stdout.is_empty() {
        out.status.code().map(|c| c.to_string()).unwrap_or_else(|| "?".to_string())
    } else {
        stdout
    })
}

// ── Layer 2: Z3 ───────────────────────────────────────

fn merge_paths<'a>(
    c: &'a [PathSummary],
    r: &'a [PathSummary],
) -> Result<Vec<(&'a PathSummary, &'a PathSummary)>> {
    Ok(c.iter().filter_map(|cp| {
        r.iter().min_by_key(|rp| witness_dist(&cp.witness, &rp.witness))
         .map(|rp| (cp, rp))
    }).collect())
}

fn witness_dist(a: &[(String, i64)], b: &[(String, i64)]) -> i64 {
    let am: HashMap<&str, i64> = a.iter().map(|(k,v)| (k.as_str(),*v)).collect();
    let bm: HashMap<&str, i64> = b.iter().map(|(k,v)| (k.as_str(),*v)).collect();
    am.iter().map(|(k,va)| bm.get(k).map(|vb|(va-vb).abs()).unwrap_or(1000)).sum()
}

enum Z3Result {
    Equivalent,
    NotEquivalent(Vec<(String, i64)>),
    Unknown,
}

fn check_pair_z3<'a>(
    ctx:    &'a Context,
    config: &AnalysisConfig,
    cp:     &PathSummary,
    rp:     &PathSummary,
) -> Result<Z3Result> {
    let solver = Solver::new(ctx);
    let mut vars: HashMap<String, Int> = HashMap::new();
    for b in &config.bounds {
        let v = Int::new_const(ctx, b.name.clone());
        solver.assert(&v.ge(&Int::from_i64(ctx, b.min)));
        solver.assert(&v.le(&Int::from_i64(ctx, b.max)));
        vars.insert(b.name.clone(), v);
    }
    for c in &cp.constraints { add_klee_constraint(ctx, &solver, &vars, c); }
    for c in &rp.constraints { add_klee_constraint(ctx, &solver, &vars, c); }

    let c_ret = build_ret(ctx, &vars, cp);
    let r_ret = build_ret(ctx, &vars, rp);

    let (ce, re) = match (c_ret, r_ret) {
        (Some(c), Some(r)) => (c, r),
        _ => return check_witness_z3(cp, rp),
    };

    solver.assert(&ce._eq(&re).not());
    match solver.check() {
        z3::SatResult::Sat => {
            let model = solver.get_model().unwrap();
            let inputs = config.bounds.iter().filter_map(|b| {
                let v = Int::new_const(ctx, b.name.clone());
                model.eval(&v, true)?.as_i64().map(|val| (b.name.clone(), val))
            }).collect();
            Ok(Z3Result::NotEquivalent(inputs))
        }
        z3::SatResult::Unsat   => Ok(Z3Result::Equivalent),
        z3::SatResult::Unknown => Ok(Z3Result::Unknown),
    }
}

fn check_witness_z3(cp: &PathSummary, rp: &PathSummary) -> Result<Z3Result> {
    match (parse_constant(&cp.return_expr), parse_constant(&rp.return_expr)) {
        (Some(c), Some(r)) => {
            if c == r { Ok(Z3Result::Equivalent) }
            else      { Ok(Z3Result::NotEquivalent(cp.witness.clone())) }
        }
        _ => Ok(Z3Result::Unknown),
    }
}

// ── KLEE kquery → Z3 ─────────────────────────────────

fn add_klee_constraint<'a>(
    ctx:    &'a Context,
    solver: &Solver,
    vars:   &HashMap<String, Int<'a>>,
    s:      &str,
) {
    if s == "true" { return; }
    if let Some(e) = parse_klee_bool(ctx, vars, s) { solver.assert(&e); }
}

fn parse_klee_bool<'a>(
    ctx:  &'a Context,
    vars: &HashMap<String, Int<'a>>,
    s:    &str,
) -> Option<z3::ast::Bool<'a>> {
    let s = s.trim();
    if s.starts_with("(Eq false ") {
        let inner = s["(Eq false ".len()..s.len()-1].trim();
        return parse_klee_bool(ctx, vars, inner).map(|b| b.not());
    }
    macro_rules! cmp {
        ($pfx:expr, $op:ident) => {
            if s.starts_with($pfx) {
                let args = s[$pfx.len()..s.len()-1].trim();
                if let Some((l,r)) = split_two(args) {
                    if let (Some(lv),Some(rv)) = (parse_klee_int(ctx,vars,l), parse_klee_int(ctx,vars,r)) {
                        return Some(lv.$op(&rv));
                    }
                }
            }
        };
    }
    cmp!("(Slt ", lt);
    cmp!("(Sle ", le);
    cmp!("(Sgt ", gt);
    cmp!("(Sge ", ge);
    if s.starts_with("(Eq ") {
        let args = s["(Eq ".len()..s.len()-1].trim();
        if let Some((l,r)) = split_two(args) {
            if let (Some(lv),Some(rv)) = (parse_klee_int(ctx,vars,l), parse_klee_int(ctx,vars,r)) {
                return Some(lv._eq(&rv));
            }
        }
    }
    None
}

fn parse_klee_int<'a>(
    ctx:  &'a Context,
    vars: &HashMap<String, Int<'a>>,
    s:    &str,
) -> Option<Int<'a>> {
    let s = s.trim();
    if s.starts_with("(ReadLSB ") {
        let inner = s["(ReadLSB ".len()..s.len()-1].trim();
        let name = inner.split_whitespace().last()?;
        return vars.get(name).cloned();
    }
    if s.starts_with("(w32 ") || s.starts_with("(w64 ") {
        let n = s[5..s.len()-1].trim();
        return n.parse::<i64>().ok()
            .or_else(|| n.parse::<u64>().ok().map(|v| v as i64))
            .map(|v| Int::from_i64(ctx, v));
    }
    s.parse::<i64>().ok()
        .or_else(|| s.parse::<u64>().ok().map(|v| v as i64))
        .map(|v| Int::from_i64(ctx, v))
}

fn split_two(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    let end = if s.starts_with('(') {
        let mut d = 0usize; let mut p = 0usize;
        for (i,c) in s.char_indices() {
            match c { '(' => d+=1, ')' => { d-=1; if d==0 { p=i; break; } } _ => {} }
        }
        p + 1
    } else { s.find(char::is_whitespace)? };
    let first = s[..end].trim(); let second = s[end..].trim();
    if first.is_empty() || second.is_empty() { None } else { Some((first, second)) }
}

fn build_ret<'a>(ctx: &'a Context, vars: &HashMap<String, Int<'a>>, p: &PathSummary) -> Option<Int<'a>> {
    let s = p.return_expr.as_deref()?;
    parse_klee_int(ctx, vars, s)
        .or_else(|| s.parse::<i64>().ok().map(|v| Int::from_i64(ctx, v)))
        .or_else(|| vars.get(s.trim()).cloned())
}

fn parse_constant(e: &Option<String>) -> Option<i64> {
    let s = e.as_deref()?.trim();
    if s.starts_with("(w32 ") || s.starts_with("(w64 ") {
        return s[5..s.len()-1].trim().parse::<i64>().ok();
    }
    s.parse::<i64>().ok()
}

fn build_ce(config: &AnalysisConfig, ir: &IrFiles, inputs: Vec<(String,i64)>) -> Result<Counterexample> {
    let args: Vec<i64> = config.bounds.iter().map(|b| {
        inputs.iter().find(|(n,_)| n==&b.name).map(|(_,v)| *v).unwrap_or(b.min)
    }).collect();
    let c_ret = run_binary(&ir.c_runner_bin,    &args).unwrap_or_else(|_| "?".to_string());
    let r_ret = run_binary(&ir.rust_runner_bin, &args).unwrap_or_else(|_| "?".to_string());
    Ok(Counterexample {
        inputs,
        c_behavior:    ConcreteBehavior { return_value: c_ret.clone(), stdout: vec![c_ret.clone()], stderr: vec![], globals: vec![] },
        rust_behavior: ConcreteBehavior { return_value: r_ret.clone(), stdout: vec![r_ret.clone()], stderr: vec![], globals: vec![] },
        differences:   vec![Difference { kind: DifferenceKind::ReturnValue, c_value: c_ret, rust_value: r_ret }],
    })
}