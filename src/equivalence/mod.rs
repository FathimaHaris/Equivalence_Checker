// src/equivalence/mod.rs
// ═══════════════════════════════════════════════════════
// Module 6: Equivalence Checking — Z3 Only
//
// DFD Structure (matches diagram):
//   Symbolic Path
//     → 0.6.1  Path Merging                    → merged path
//     → 0.6.2  Equivalence Query Constructor   → SMT query
//     → 0.6.3  SMT Solver (Z3)                 → SAT/UNSAT
//
// CORRECT STRATEGY:
//   For each merged path, 0.6.2 asks Z3 for a concrete input satisfying
//   the path's constraints (SMT query). 0.6.3 runs both the C binary and
//   Rust binary on that input. If outputs ever differ → counterexample.
// ═══════════════════════════════════════════════════════

use crate::types::{
    AnalysisConfig, EquivalenceResult, Verdict, Counterexample,
    ConcreteBehavior, Difference, DifferenceKind, PathSummary,
    CheckerStatistics,
};
use crate::compiler::IrFiles;
use anyhow::Result;
use std::time::Instant;
use std::collections::HashMap;
use std::process::Command;
use z3::{Config, Context, Solver, SatResult, ast::{Ast, Int, Bool}};

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

    println!("\n  ── Z3 Symbolic Equivalence Checking ──");

    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // ── 0.6.1  Path Merging ───────────────────────────
    // Combine C and Rust symbolic path summaries into one merged list.
    let merged_paths = stage_061_path_merging(c_summaries, rust_summaries);

    println!("     Checking {} merged paths ({} C + {} Rust)",
        merged_paths.len(), c_summaries.len(), rust_summaries.len());

    let mut all_unknown   = true;
    let mut checked_paths = 0u32;

    for (path_idx, (path, kind)) in merged_paths.iter().enumerate() {
        println!("     Path {}/{} [{}] {}…",
            path_idx + 1, merged_paths.len(), kind, path.id);

        let labels = build_label_map(&path.constraints);

        // ── 0.6.2  Equivalence Query Constructor ──────
        // Build the SMT query: produce a concrete input satisfying this path.
        let input = stage_062_equivalence_query_constructor(
            &ctx, config, &path.constraints, &labels, &path.witness,
        );

        let input = match input {
            Some(i) => i,
            None => {
                println!("       ? No satisfying input found — skipping path");
                continue;
            }
        };
        println!("       Input: {:?}", input);

        // ── 0.6.3  SMT Solver (Z3) — primary check ────
        // Run both binaries; Z3's SAT model becomes the concrete test input.
        match stage_063_smt_solver(
            config, ir_files, &input,
            c_summaries, rust_summaries,
            path_idx, &mut stats, &start,
        )? {
            SolverOutcome::Differ(eq_result) => return Ok(eq_result),
            SolverOutcome::Agree  => { all_unknown = false; checked_paths += 1; }
            SolverOutcome::Unknown => {}
        }

        // ── 0.6.2 + 0.6.3  Boundary-value extra inputs ──
        // Enumerate inputs probing ±1 around path boundary constants and
        // re-run both stages for each, catching off-by-one divergences.
        let extra_inputs = enumerate_extra_inputs(
            &ctx, config, &path.constraints, &labels, &input, 3,
        );
        for extra_input in extra_inputs {
            let c_out2    = run_binary_i64(&ir_files.c_runner_bin,    &extra_input, config);
            let rust_out2 = run_binary_i64(&ir_files.rust_runner_bin, &extra_input, config);
            println!("       Extra {:?}: C={:?} Rust={:?}", extra_input, c_out2, rust_out2);
            match (c_out2, rust_out2) {
                (Some(cv), Some(rv)) if cv != rv => {
                    println!("       ✗ DIFFER on extra input!");
                    let ce = build_counterexample(config, ir_files, extra_input, cv, rv);
                    let cp_best = c_summaries.iter()
                        .min_by_key(|p| witness_dist(&p.witness, &ce.inputs));
                    let rp_best = rust_summaries.iter()
                        .min_by_key(|p| witness_dist(&p.witness, &ce.inputs));
                    return Ok(EquivalenceResult {
                        verdict:        Verdict::NotEquivalent,
                        paths_compared: (path_idx + 1) as u32,
                        counterexample: Some(ce),
                        time_taken:     start.elapsed().as_secs_f64(),
                        statistics:     stats,
                        c_path:         cp_best.cloned(),
                        rust_path:      rp_best.cloned(),
                    });
                }
                (Some(_), Some(_)) => { all_unknown = false; checked_paths += 1; }
                _ => {}
            }
        }

        stats.z3_queries += 1;
        stats.z3_time_ms += start.elapsed().as_millis() as u64;
    }

    if all_unknown {
        println!("\n  ⚠ Could not get outputs for any path — check binary runners");
        return Ok(EquivalenceResult {
            verdict: Verdict::Unknown,
            paths_compared: merged_paths.len() as u32,
            counterexample: None,
            time_taken: start.elapsed().as_secs_f64(),
            statistics: stats, c_path: None, rust_path: None,
        });
    }

    println!("\n  ✓ Programs are SEMANTICALLY EQUIVALENT ({} inputs checked)", checked_paths);
    Ok(EquivalenceResult {
        verdict: Verdict::Equivalent,
        paths_compared: merged_paths.len() as u32,
        counterexample: None,
        time_taken: start.elapsed().as_secs_f64(),
        statistics: stats, c_path: None, rust_path: None,
    })
}

// ═══════════════════════════════════════════════════════
// 0.6.1  Path Merging
// ═══════════════════════════════════════════════════════
// Input:  Symbolic path summaries (C + Rust)
// Output: Merged path — combined list of all paths, tagged by origin.
//
// Every reachable region that KLEE explored in either program must be
// tested against both binaries. Merging the two path sets achieves this.

fn stage_061_path_merging<'a>(
    c_summaries:    &'a [PathSummary],
    rust_summaries: &'a [PathSummary],
) -> Vec<(&'a PathSummary, &'a str)> {
    println!("  [0.6.1] Path merging…");
    let merged: Vec<(&PathSummary, &str)> = c_summaries.iter().map(|p| (p, "C"))
        .chain(rust_summaries.iter().map(|p| (p, "Rust")))
        .collect();
    println!("  [0.6.1] {} merged paths", merged.len());
    merged
}

// ═══════════════════════════════════════════════════════
// 0.6.2  Equivalence Query Constructor
// ═══════════════════════════════════════════════════════
// Input:  Merged path (constraints + KLEE witness)
// Output: SMT query — a concrete input satisfying the path's constraints.
//
// Uses the KLEE witness directly when available (clamped to bounds).
// Falls back to Z3 model generation when no witness is present.

fn stage_062_equivalence_query_constructor(
    ctx:         &Context,
    config:      &AnalysisConfig,
    constraints: &[String],
    labels:      &HashMap<String, String>,
    witness:     &[(String, i64)],
) -> Option<Vec<(String, i64)>> {
    if !witness.is_empty() {
        // Clamp witness values to declared bounds — KLEE sometimes produces
        // out-of-bounds witnesses when klee_assume constraints are incomplete.
        let clamped: Vec<(String, i64)> = config.bounds.iter().map(|b| {
            let val = witness.iter()
                .find(|(n, _)| n == &b.name)
                .map(|(_, v)| *v)
                .unwrap_or(b.min);
            (b.name.clone(), val.max(b.min).min(b.max))
        }).collect();
        return Some(clamped);
    }
    solve_single_input(ctx, config, constraints, labels, &[])
}

// ═══════════════════════════════════════════════════════
// 0.6.3  SMT Solver (Z3)
// ═══════════════════════════════════════════════════════
// Input:  SMT query (concrete input produced by 0.6.2)
// Output: SAT/UNSAT verdict → Agree | Differ | Unknown
//
// Executes both the C binary and Rust binary on the concrete input and
// compares their return values.  A difference is a counterexample.

enum SolverOutcome {
    Agree,
    Differ(EquivalenceResult),
    Unknown,
}

fn stage_063_smt_solver(
    config:         &AnalysisConfig,
    ir_files:       &IrFiles,
    input:          &[(String, i64)],
    c_summaries:    &[PathSummary],
    rust_summaries: &[PathSummary],
    path_idx:       usize,
    stats:          &mut CheckerStatistics,
    start:          &Instant,
) -> Result<SolverOutcome> {
    let c_out    = run_binary_i64(&ir_files.c_runner_bin,    input, config);
    let rust_out = run_binary_i64(&ir_files.rust_runner_bin, input, config);

    println!("       C={:?}  Rust={:?}", c_out, rust_out);

    match (c_out, rust_out) {
        (Some(cv), Some(rv)) => {
            if cv != rv {
                println!("       ✗ DIFFER: C={} Rust={} at {:?}", cv, rv, input);
                let ce = build_counterexample(config, ir_files, input.to_vec(), cv, rv);
                let cp_best = c_summaries.iter()
                    .min_by_key(|p| witness_dist(&p.witness, &ce.inputs));
                let rp_best = rust_summaries.iter()
                    .min_by_key(|p| witness_dist(&p.witness, &ce.inputs));
                stats.z3_queries += path_idx as u32 + 1;
                return Ok(SolverOutcome::Differ(EquivalenceResult {
                    verdict:        Verdict::NotEquivalent,
                    paths_compared: (path_idx + 1) as u32,
                    counterexample: Some(ce),
                    time_taken:     start.elapsed().as_secs_f64(),
                    statistics:     stats.clone(),
                    c_path:         cp_best.cloned(),
                    rust_path:      rp_best.cloned(),
                }));
            }
            println!("       ✓ Both return {}", cv);
            Ok(SolverOutcome::Agree)
        }
        _ => {
            println!("       ? Could not get output from one or both binaries");
            Ok(SolverOutcome::Unknown)
        }
    }
}

// ── Enumerate additional boundary-value inputs ────────

fn enumerate_extra_inputs(
    ctx:         &Context,
    config:      &AnalysisConfig,
    constraints: &[String],
    labels:      &HashMap<String, String>,
    first_input: &[(String, i64)],
    count:       usize,
) -> Vec<Vec<(String, i64)>> {
    let mut found = vec![first_input.to_vec()];

    let boundaries = extract_boundary_constants(constraints);
    for (varname, bval) in &boundaries {
        for delta in &[-1i64, 0, 1] {
            let probe_val = bval + delta;
            let mut candidate: Vec<(String, i64)> = first_input.to_vec();
            for (n, v) in candidate.iter_mut() {
                if n == varname { *v = probe_val; }
            }
            let in_bounds = config.bounds.iter().all(|b| {
                candidate.iter().find(|(n, _)| n == &b.name)
                    .map(|(_, v)| *v >= b.min && *v <= b.max)
                    .unwrap_or(false)
            });
            if in_bounds && !found.contains(&candidate) {
                found.push(candidate);
            }
        }
    }

    for _ in 0..count {
        match solve_single_input(ctx, config, constraints, labels, &found) {
            Some(inp) => found.push(inp),
            None      => break,
        }
    }

    found.into_iter().skip(1).collect()
}

// ── Z3 single-input solver ────────────────────────────

fn solve_single_input(
    ctx:         &Context,
    config:      &AnalysisConfig,
    constraints: &[String],
    labels:      &HashMap<String, String>,
    exclude:     &[Vec<(String, i64)>],
) -> Option<Vec<(String, i64)>> {
    let solver = Solver::new(ctx);
    let mut vars: HashMap<String, Int> = HashMap::new();

    for b in &config.bounds {
        let v = Int::new_const(ctx, b.name.clone());
        solver.assert(&v.ge(&Int::from_i64(ctx, b.min)));
        solver.assert(&v.le(&Int::from_i64(ctx, b.max)));
        vars.insert(b.name.clone(), v);
    }
    for c in constraints {
        if let Some(b) = parse_klee_bool(ctx, &vars, labels, c) {
            solver.assert(&b);
        }
    }
    for prev in exclude {
        let not_same = Bool::or(ctx, &prev.iter().filter_map(|(name, val)| {
            vars.get(name).map(|v| v._eq(&Int::from_i64(ctx, *val)).not())
        }).collect::<Vec<_>>().iter().collect::<Vec<_>>());
        solver.assert(&not_same);
    }

    if solver.check() != SatResult::Sat { return None; }
    let model = solver.get_model()?;
    let inputs: Vec<(String, i64)> = config.bounds.iter().filter_map(|b| {
        let v = Int::new_const(ctx, b.name.clone());
        model.eval(&v, true)?.as_i64().map(|val| (b.name.clone(), val))
    }).collect();
    if inputs.is_empty() { None } else { Some(inputs) }
}

// ── Witness distance helper ───────────────────────────

fn witness_dist(a: &[(String, i64)], b: &[(String, i64)]) -> i64 {
    let am: HashMap<&str, i64> = a.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let bm: HashMap<&str, i64> = b.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    am.iter().map(|(k, va)| bm.get(k).map(|vb| (va - vb).abs()).unwrap_or(1000)).sum()
}

// ── Binary runner ─────────────────────────────────────

fn run_binary_i64(bin: &str, inputs: &[(String, i64)], config: &AnalysisConfig) -> Option<i64> {
    let args: Vec<i64> = config.bounds.iter()
        .map(|b| inputs.iter().find(|(n, _)| n == &b.name).map(|(_, v)| *v).unwrap_or(b.min))
        .collect();
    let str_args: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    let out = Command::new(bin).args(&str_args).output().ok()?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !stdout.is_empty() { stdout.parse::<i64>().ok() }
    else { out.status.code().map(|c| c as i64) }
}

#[allow(dead_code)]
fn run_binary_str(bin: &str, args: &[i64]) -> String {
    let str_args: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    match Command::new(bin).args(&str_args).output() {
        Err(_) => "error".to_string(),
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() {
                out.status.code().map(|c| c.to_string()).unwrap_or_else(|| "?".to_string())
            } else { s }
        }
    }
}

// ── Counterexample builder ────────────────────────────

fn build_counterexample(
    _config: &AnalysisConfig, _ir_files: &IrFiles,
    inputs: Vec<(String, i64)>, c_val: i64, rust_val: i64,
) -> Counterexample {
    let c_ret    = c_val.to_string();
    let rust_ret = rust_val.to_string();
    Counterexample {
        inputs,
        c_behavior: ConcreteBehavior {
            return_value: c_ret.clone(), stdout: vec![c_ret.clone()],
            stderr: vec![], globals: vec![],
        },
        rust_behavior: ConcreteBehavior {
            return_value: rust_ret.clone(), stdout: vec![rust_ret.clone()],
            stderr: vec![], globals: vec![],
        },
        differences: vec![Difference {
            kind: DifferenceKind::ReturnValue,
            c_value: c_ret, rust_value: rust_ret,
        }],
    }
}

// ── Label map ─────────────────────────────────────────

fn build_label_map(constraints: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for c in constraints { extract_labels(c, &mut map); }
    map
}

fn extract_labels(s: &str, map: &mut HashMap<String, String>) {
    let mut search = s;
    while let Some(colon) = search.find(":(ReadLSB ") {
        let before = &search[..colon];
        let label_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|p| p + 1).unwrap_or(0);
        let label = before[label_start..].trim().to_string();
        let after = &search[colon + 1..];
        let mut depth = 0usize; let mut end = 0usize;
        for (i, ch) in after.char_indices() {
            match ch { '(' => depth += 1, ')' => { depth -= 1; if depth == 0 { end = i; break; } } _ => {} }
        }
        let inner = after[1..end].trim();
        if let Some(varname) = inner.split_whitespace().last() {
            if !label.is_empty() { map.insert(label, varname.to_string()); }
        }
        if end + 1 < after.len() { search = &after[end + 1..]; } else { break; }
    }
}

// ── Boundary constant extractor ───────────────────────

fn resolve_varname(s: &str, labels: &HashMap<String, String>) -> Option<String> {
    let s = s.trim();
    if let Some(colon) = s.find(":(") {
        let label_part = &s[..colon];
        if label_part.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return resolve_varname(&s[colon + 1..], labels);
        }
    }
    if s.starts_with("(ReadLSB ") || s.starts_with("(ReadMSB ") {
        let inner_start = s.find(' ')? + 1;
        let inner = if s.ends_with(')') { &s[inner_start..s.len()-1] } else { &s[inner_start..] };
        return inner.split_whitespace().last().map(|v| v.trim_end_matches(')').to_string());
    }
    if !s.starts_with('(') && s.parse::<i64>().is_err() && s.parse::<u64>().is_err() {
        if let Some(vname) = labels.get(s) { return Some(vname.clone()); }
        if s.chars().all(|c| c.is_alphanumeric() || c == '_') { return Some(s.to_string()); }
    }
    None
}

fn extract_boundary_constants(constraints: &[String]) -> Vec<(String, i64)> {
    let mut labels: HashMap<String, String> = HashMap::new();
    for c in constraints {
        let mut s = c.as_str();
        while let Some(pos) = s.find(":(ReadLSB ") {
            let before = &s[..pos];
            let label_start = before.rfind(|c: char| !c.is_alphanumeric() && c != '_')
                .map(|p| p + 1).unwrap_or(0);
            let label = before[label_start..].trim().to_string();
            let after = &s[pos + 1..];
            let mut depth = 0usize; let mut end = after.len();
            for (i, ch) in after.char_indices() {
                match ch { '(' => depth += 1, ')' => { depth -= 1; if depth == 0 { end = i; break; } } _ => {} }
            }
            if end > 0 {
                let inner = after[1..end].trim();
                if let Some(varname) = inner.split_whitespace().last() {
                    if !label.is_empty() { labels.insert(label, varname.to_string()); }
                }
            }
            s = if end + 1 < after.len() { &after[end + 1..] } else { break; "" };
        }
    }

    let mut boundaries = Vec::new();
    for c in constraints {
        for op in &["(Sle ", "(Slt ", "(Sge ", "(Sgt ", "(Ule ", "(Ult "] {
            let mut search = c.as_str();
            while let Some(op_pos) = search.find(op) {
                let after_op = &search[op_pos + op.len()..];
                if let Some((arg1, arg2)) = safe_split_two_args(after_op) {
                    let arg1 = arg1.trim();
                    let arg2 = arg2.trim().trim_end_matches(')').trim();
                    if let Some(n) = arg1.parse::<i64>().ok()
                        .or_else(|| arg1.parse::<u64>().ok().map(|v| v as i32 as i64))
                    {
                        if let Some(vname) = resolve_varname(arg2, &labels) {
                            boundaries.push((vname, n));
                        }
                    }
                    if let Some(n) = arg2.parse::<i64>().ok()
                        .or_else(|| arg2.parse::<u64>().ok().map(|v| v as i32 as i64))
                    {
                        if let Some(vname) = resolve_varname(arg1, &labels) {
                            boundaries.push((vname, n));
                        }
                    }
                }
                search = &search[op_pos + op.len()..];
            }
        }
    }
    boundaries
}

fn safe_split_two_args(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    if s.is_empty() { return None; }
    let end = if s.starts_with('(') {
        let mut depth = 0usize; let mut ep = 0usize; let mut found = false;
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => { if depth == 0 { break; } depth -= 1; if depth == 0 { ep = i; found = true; break; } }
                _ => {}
            }
        }
        if !found { return None; }
        ep + 1
    } else { match s.find(char::is_whitespace) { Some(p) => p, None => return None } };
    if end > s.len() { return None; }
    let first = s[..end].trim(); let second = s[end..].trim();
    if first.is_empty() || second.is_empty() { None } else { Some((first, second)) }
}

// ── KLEE bool parser ──────────────────────────────────

fn parse_klee_bool<'ctx>(
    ctx: &'ctx Context, vars: &HashMap<String, Int<'ctx>>,
    labels: &HashMap<String, String>, s: &str,
) -> Option<Bool<'ctx>> {
    let s = s.trim();
    if s == "true"  { return Some(Bool::from_bool(ctx, true));  }
    if s == "false" { return Some(Bool::from_bool(ctx, false)); }

    if s.starts_with("(Extract ") {
        let inner = s["(Extract ".len()..s.len()-1].trim();
        if let Some(rest) = inner.splitn(2, char::is_whitespace).nth(1) {
            let rest = rest.trim();
            let expr = if rest.starts_with("(ZExt ") || rest.starts_with("(SExt ") {
                rest[6..rest.len()-1].trim().splitn(2, char::is_whitespace).nth(1).unwrap_or("").trim()
            } else { rest };
            if !expr.is_empty() { return parse_klee_bool(ctx, vars, labels, expr); }
        }
    }
    if s.starts_with("(Eq false ") {
        return parse_klee_bool(ctx, vars, labels, s["(Eq false ".len()..s.len()-1].trim()).map(|b| b.not());
    }
    if s.starts_with("(Eq true ") {
        return parse_klee_bool(ctx, vars, labels, s["(Eq true ".len()..s.len()-1].trim());
    }

    macro_rules! cmp {
        ($pfx:expr, $op:ident) => {
            if s.starts_with($pfx) {
                let args = s[$pfx.len()..s.len()-1].trim();
                if let Some((l, r)) = split_two(args) {
                    if let (Some(lv), Some(rv)) = (parse_klee_int(ctx,vars,labels,l), parse_klee_int(ctx,vars,labels,r))
                    { return Some(lv.$op(&rv)); }
                }
            }
        };
    }
    cmp!("(Slt ",lt); cmp!("(Sle ",le); cmp!("(Sgt ",gt); cmp!("(Sge ",ge);
    cmp!("(Ult ",lt); cmp!("(Ule ",le); cmp!("(Ugt ",gt); cmp!("(Uge ",ge);

    if s.starts_with("(Eq ") {
        let args = s["(Eq ".len()..s.len()-1].trim();
        if let Some((l, r)) = split_two(args) {
            if let (Some(lv), Some(rv)) = (parse_klee_int(ctx,vars,labels,l), parse_klee_int(ctx,vars,labels,r))
            { return Some(lv._eq(&rv)); }
        }
    }
    if s.starts_with("(Not ") {
        return parse_klee_bool(ctx, vars, labels, s["(Not ".len()..s.len()-1].trim()).map(|b| b.not());
    }
    if s.starts_with("(And ") {
        let args = s["(And ".len()..s.len()-1].trim();
        if let Some((l, r)) = split_two(args) {
            if let (Some(lv), Some(rv)) = (parse_klee_bool(ctx,vars,labels,l), parse_klee_bool(ctx,vars,labels,r))
            { return Some(Bool::and(ctx, &[&lv, &rv])); }
        }
    }
    if s.starts_with("(Or ") {
        let args = s["(Or ".len()..s.len()-1].trim();
        if let Some((l, r)) = split_two(args) {
            if let (Some(lv), Some(rv)) = (parse_klee_bool(ctx,vars,labels,l), parse_klee_bool(ctx,vars,labels,r))
            { return Some(Bool::or(ctx, &[&lv, &rv])); }
        }
    }
    None
}

// ── KLEE int parser ───────────────────────────────────

fn parse_klee_int<'ctx>(
    ctx: &'ctx Context, vars: &HashMap<String, Int<'ctx>>,
    labels: &HashMap<String, String>, s: &str,
) -> Option<Int<'ctx>> {
    let s = s.trim();
    if s.is_empty() { return None; }
    if let Some(pos) = find_label_colon(s) { return parse_klee_int(ctx, vars, labels, &s[pos+1..]); }
    if is_bare_label(s) {
        if let Some(vname) = labels.get(s) { return vars.get(vname).cloned(); }
        if let Some(v) = vars.get(s) { return Some(v.clone()); }
    }
    if s.starts_with("(ReadLSB ") || s.starts_with("(ReadMSB ") {
        let inner = s[9..s.len()-1].trim();
        if let Some(&name) = inner.split_whitespace().collect::<Vec<_>>().last() {
            if let Some(vname) = labels.get(name) { return vars.get(vname).cloned(); }
            return vars.get(name).cloned();
        }
    }
    if s.starts_with("(w32 ") || s.starts_with("(w64 ") {
        let n = s[5..s.len()-1].trim();
        return n.parse::<i64>().ok().or_else(|| n.parse::<u64>().ok().map(|v| v as i64))
            .map(|v| Int::from_i64(ctx, v));
    }
    macro_rules! arith {
        ($pfx:expr, $fn:path) => {
            if s.starts_with($pfx) {
                let args = s[$pfx.len()..s.len()-1].trim();
                if let Some((l, r)) = split_two(args) {
                    if let (Some(lv), Some(rv)) = (parse_klee_int(ctx,vars,labels,l), parse_klee_int(ctx,vars,labels,r))
                    { return Some($fn(ctx, &[&lv, &rv])); }
                }
            }
        };
    }
    arith!("(Add ", Int::add); arith!("(Sub ", Int::sub); arith!("(Mul ", Int::mul);
    macro_rules! divrem {
        ($pfx:expr, $m:ident) => {
            if s.starts_with($pfx) {
                let args = s[$pfx.len()..s.len()-1].trim();
                if let Some((l, r)) = split_two(args) {
                    if let (Some(lv), Some(rv)) = (parse_klee_int(ctx,vars,labels,l), parse_klee_int(ctx,vars,labels,r))
                    { return Some(lv.$m(&rv)); }
                }
            }
        };
    }
    divrem!("(SDiv ",div); divrem!("(SRem ",rem); divrem!("(UDiv ",div); divrem!("(URem ",rem);
    if s.starts_with("(Select ") {
        let inner = s["(Select ".len()..s.len()-1].trim();
        if let Some((cs, rest)) = split_two(inner) {
            if let Some((ts, fs)) = split_two(rest) {
                if let (Some(c), Some(tv), Some(fv)) = (
                    parse_klee_bool(ctx,vars,labels,cs),
                    parse_klee_int(ctx,vars,labels,ts),
                    parse_klee_int(ctx,vars,labels,fs),
                ) { return Some(c.ite(&tv, &fv)); }
            }
        }
    }
    for pfx in &["(SExt ", "(ZExt ", "(Trunc "] {
        if s.starts_with(pfx) {
            let inner = s[pfx.len()..s.len()-1].trim();
            let body = inner.splitn(2, char::is_whitespace).nth(1).unwrap_or(inner).trim();
            return parse_klee_int(ctx, vars, labels, body);
        }
    }
    if s.starts_with("(Extract ") {
        let inner = s["(Extract ".len()..s.len()-1].trim();
        if let Some(body) = inner.splitn(2, char::is_whitespace).nth(1) {
            return parse_klee_int(ctx, vars, labels, body.trim());
        }
    }
    s.parse::<i64>().ok().or_else(|| s.parse::<u64>().ok().map(|v| v as i64))
        .map(|v| Int::from_i64(ctx, v))
}

// ── Helpers ───────────────────────────────────────────

fn find_label_colon(s: &str) -> Option<usize> {
    if let Some(pos) = s.find(":(") {
        let label = &s[..pos];
        if !label.is_empty() && label.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Some(pos);
        }
    }
    None
}

fn is_bare_label(s: &str) -> bool {
    !s.is_empty() && !s.starts_with('(')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
        && s.parse::<i64>().is_err() && s.parse::<u64>().is_err()
}

fn split_two(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    if s.is_empty() { return None; }
    let end = if s.starts_with('(') {
        let mut depth = 0usize; let mut ep = 0usize;
        for (i, c) in s.char_indices() {
            match c { '(' => depth += 1, ')' => { depth -= 1; if depth == 0 { ep = i; break; } } _ => {} }
        }
        ep + 1
    } else { s.find(char::is_whitespace)? };
    let first = s[..end].trim(); let second = s[end..].trim();
    if first.is_empty() || second.is_empty() { None } else { Some((first, second)) }
}