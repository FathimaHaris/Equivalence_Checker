// src/equivalence/mod.rs
use crate::types::{
    AnalysisConfig, EquivalenceResult, Verdict, Counterexample,
    ConcreteBehavior, Difference, DifferenceKind, PathSummary,
    CheckerStatistics, ReturnKind, EquivalenceDetail, DivergingInput, ParamType,
};
use crate::compiler::IrFiles;
use anyhow::Result;
use std::time::Instant;
use std::collections::HashMap;
use std::process::Command;
use z3::{Config, Context, Solver, SatResult, ast::{Ast, Int, Bool}};

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

    println!("\n  -- Equivalence Checking (KLEE + Concrete Execution) --");

    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // Collect test inputs from three sources:
    // (a) KLEE witnesses embedded in path summaries
    // (b) Z3-solved inputs satisfying each path's constraints
    // (c) Systematic boundary + interior sweep

    let mut all_inputs: Vec<Vec<(String, i64)>> = Vec::new();

    // (a) witnesses
    for ps in c_summaries.iter().chain(rust_summaries.iter()) {
        if !ps.witness.is_empty() {
            push_unique(&mut all_inputs, clamp_to_bounds(&ps.witness, config));
        }
    }

    // (b) Z3: up to 4 distinct inputs per path
    for ps in c_summaries.iter().chain(rust_summaries.iter()) {
        let mut seen: Vec<Vec<(String, i64)>> = Vec::new();
        for _ in 0..4 {
            if let Some(inp) = solve_single_input(
                &ctx, config, &ps.constraints, &ps.label_map, &seen,
            ) {
                seen.push(inp.clone());
                push_unique(&mut all_inputs, inp);
            }
        }
    }

    // (c) boundary + interior sweep
    for inp in generate_boundary_inputs(config) {
        push_unique(&mut all_inputs, inp);
    }

    println!("     Generated {} test inputs", all_inputs.len());

    let mut checked = 0u32;
    let mut detail  = EquivalenceDetail::default();

    for input in &all_inputs {
        println!("     Testing {:?}", input);
        let c_out    = run_binary(&ir_files.c_runner_bin,    input, config);
        let rust_out = run_binary(&ir_files.rust_runner_bin, input, config);

        match (&c_out, &rust_out) {
            (BinaryOutput::Error, _) | (_, BinaryOutput::Error) => {
                println!("       ? runner error -- skipping");
                continue;
            }
            _ => {}
        }

        detail.inputs_tested += 1;
        checked += 1;
        stats.merged_pairs += 1;

        println!("       C={}  Rust={}", c_out.to_string_repr(), rust_out.to_string_repr());

        if c_out.differs_from(&rust_out) {
            println!("       ✗ DIFFER -- counterexample found");
            detail.return_value_match = Some(false);

            let input_strings: Vec<(String, String)> =
                input.iter().map(|(n, v)| (n.clone(), v.to_string())).collect();

            let ce = Counterexample {
                inputs: input.clone(),
                input_strings,
                c_behavior: ConcreteBehavior {
                    return_value: c_out.to_string_repr(),
                    ..Default::default()
                },
                rust_behavior: ConcreteBehavior {
                    return_value: rust_out.to_string_repr(),
                    ..Default::default()
                },
                differences: vec![Difference {
                    kind: DifferenceKind::ReturnValue,
                    c_value: c_out.to_string_repr(),
                    rust_value: rust_out.to_string_repr(),
                }],
            };

            let cp = c_summaries.iter()
                .min_by_key(|p| witness_dist(&p.witness, input));
            let rp = rust_summaries.iter()
                .min_by_key(|p| witness_dist(&p.witness, input));

            return Ok(EquivalenceResult {
                verdict: Verdict::NotEquivalent,
                paths_compared: checked,
                counterexample: Some(ce),
                time_taken: start.elapsed().as_secs_f64(),
                statistics: stats,
                c_path: cp.cloned(),
                rust_path: rp.cloned(),
            });
        }

        println!("       ✓ Both return {}", c_out.to_string_repr());
    }

    if checked == 0 {
        println!("\n  ⚠ Could not execute any inputs -- check runner binaries");
        return Ok(EquivalenceResult {
            verdict: Verdict::Unknown,
            paths_compared: 0,
            counterexample: None,
            time_taken: start.elapsed().as_secs_f64(),
            statistics: stats,
            c_path: None,
            rust_path: None,
        });
    }

    println!("\n  ✓ Programs are SEMANTICALLY EQUIVALENT ({} inputs checked)", checked);
    Ok(EquivalenceResult {
        verdict: Verdict::Equivalent,
        paths_compared: checked,
        counterexample: None,
        time_taken: start.elapsed().as_secs_f64(),
        statistics: stats,
        c_path: None,
        rust_path: None,
    })
}

// ── BinaryOutput ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum BinaryOutput {
    Int(i64),
    Float(f64),
    Void,
    Error,
}

impl BinaryOutput {
    fn differs_from(&self, other: &BinaryOutput) -> bool {
        match (self, other) {
            (BinaryOutput::Int(a),   BinaryOutput::Int(b))   => a != b,
            (BinaryOutput::Float(a), BinaryOutput::Float(b)) => {
                let diff = (a - b).abs();
                diff > 1e-9 * a.abs().max(b.abs()).max(1.0)
            }
            (BinaryOutput::Void, BinaryOutput::Void) => false,
            _ => true,
        }
    }

    fn to_string_repr(&self) -> String {
        match self {
            BinaryOutput::Int(v)   => v.to_string(),
            BinaryOutput::Float(v) => format!("{:.17e}", v),
            BinaryOutput::Void     => "void".into(),
            BinaryOutput::Error    => "error".into(),
        }
    }
}

// ── Runner ────────────────────────────────────────────────────────────────────

fn run_binary(
    bin:    &str,
    inputs: &[(String, i64)],
    config: &AnalysisConfig,
) -> BinaryOutput {
    // Pass args in declaration order (must match how the runner was generated)
    let str_args: Vec<String> = config.bounds.iter().map(|b| {
        inputs.iter()
            .find(|(n, _)| n == &b.name)
            .map(|(_, v)| v.to_string())
            .unwrap_or_else(|| "0".into())
    }).collect();

    let out = match Command::new(bin).args(&str_args).output() {
        Ok(o)  => o,
        Err(e) => {
            eprintln!("    [runner] exec failed {}: {}", bin, e);
            return BinaryOutput::Error;
        }
    };

    // C runner:    printf("%d\n", r)
    // Rust runner: println!("{}", r)
    let stdout = String::from_utf8_lossy(&out.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if let Ok(n) = trimmed.parse::<i64>() { return BinaryOutput::Int(n); }
        if trimmed == "true"  { return BinaryOutput::Int(1); }
        if trimmed == "false" { return BinaryOutput::Int(0); }
    }

    // Last resort: exit code (unreliable for negative/large values)
    if let Some(code) = out.status.code() {
        return BinaryOutput::Int(code as i64);
    }
    BinaryOutput::Error
}

// ── Input generators ──────────────────────────────────────────────────────────

fn clamp_to_bounds(witness: &[(String, i64)], config: &AnalysisConfig) -> Vec<(String, i64)> {
    config.bounds.iter().map(|b| {
        let v = witness.iter()
            .find(|(n, _)| n == &b.name)
            .map(|(_, v)| *v)
            .unwrap_or(b.min);
        (b.name.clone(), v.max(b.min).min(b.max))
    }).collect()
}

fn push_unique(list: &mut Vec<Vec<(String, i64)>>, input: Vec<(String, i64)>) {
    if !list.contains(&input) { list.push(input); }
}

/// Cartesian product of candidate values for each variable (capped at 500).
/// Candidates per variable: min, min+1, q1, mid, q3, max-1, max, 0, 1, -1
fn generate_boundary_inputs(config: &AnalysisConfig) -> Vec<Vec<(String, i64)>> {
    let candidates: Vec<Vec<i64>> = config.bounds.iter().map(|b| {
        let mut vals = vec![b.min, b.max];
        let mid = b.min + (b.max - b.min) / 2;
        let q1  = b.min + (b.max - b.min) / 4;
        let q3  = b.min + 3 * (b.max - b.min) / 4;
        vals.push(mid);
        vals.push(q1);
        vals.push(q3);
        if b.min + 1 <= b.max { vals.push(b.min + 1); }
        if b.max - 1 >= b.min { vals.push(b.max - 1); }
        for v in [0i64, 1, -1] {
            if v >= b.min && v <= b.max { vals.push(v); }
        }
        vals.sort();
        vals.dedup();
        vals
    }).collect();

    let mut result: Vec<Vec<(String, i64)>> = vec![vec![]];
    for (b, vals) in config.bounds.iter().zip(candidates.iter()) {
        let mut next = Vec::new();
        'outer: for prefix in &result {
            for &v in vals {
                let mut row = prefix.clone();
                row.push((b.name.clone(), v));
                next.push(row);
                if next.len() >= 500 { break 'outer; }
            }
        }
        result = next;
    }
    result
}

// ── Z3 solver ─────────────────────────────────────────────────────────────────

fn solve_single_input(
    ctx:         &Context,
    config:      &AnalysisConfig,
    constraints: &[String],
    labels:      &HashMap<String, String>,
    exclude:     &[Vec<(String, i64)>],
) -> Option<Vec<(String, i64)>> {
    let solver = Solver::new(ctx);
    let mut int_vars: HashMap<String, Int> = HashMap::new();

    for b in &config.bounds {
        let v = Int::new_const(ctx, b.name.clone());
        solver.assert(&v.ge(&Int::from_i64(ctx, b.min)));
        solver.assert(&v.le(&Int::from_i64(ctx, b.max)));
        int_vars.insert(b.name.clone(), v);
    }

    for c in constraints {
        if let Some(b) = parse_klee_bool(ctx, &int_vars, labels, c) {
            solver.assert(&b);
        }
    }

    for prev in exclude {
        let clauses: Vec<Bool> = prev.iter().filter_map(|(name, val)| {
            int_vars.get(name).map(|v| v._eq(&Int::from_i64(ctx, *val)).not())
        }).collect();
        if !clauses.is_empty() {
            let refs: Vec<&Bool> = clauses.iter().collect();
            solver.assert(&Bool::or(ctx, &refs));
        }
    }

    if solver.check() != SatResult::Sat { return None; }
    let model = solver.get_model()?;

    let inputs: Vec<(String, i64)> = config.bounds.iter().filter_map(|b| {
        let v = Int::new_const(ctx, b.name.clone());
        model.eval(&v, true)?.as_i64().map(|val| (b.name.clone(), val))
    }).collect();

    if inputs.is_empty() { None } else { Some(inputs) }
}

// ── Witness distance ──────────────────────────────────────────────────────────

fn witness_dist(a: &[(String, i64)], b: &[(String, i64)]) -> i64 {
    let am: HashMap<&str, i64> = a.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let bm: HashMap<&str, i64> = b.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    am.iter().map(|(k, va)| bm.get(k).map(|vb| (va - vb).abs()).unwrap_or(1000)).sum()
}

// ── KLEE SMT parsers ──────────────────────────────────────────────────────────

fn parse_klee_bool<'ctx>(
    ctx:    &'ctx Context,
    vars:   &HashMap<String, Int<'ctx>>,
    labels: &HashMap<String, String>,
    s:      &str,
) -> Option<Bool<'ctx>> {
    let s = s.trim();
    if s == "true"  { return Some(Bool::from_bool(ctx, true));  }
    if s == "false" { return Some(Bool::from_bool(ctx, false)); }

    if s.starts_with("(Extract ") {
        let inner = s["(Extract ".len()..s.len()-1].trim();
        if let Some(rest) = inner.splitn(2, char::is_whitespace).nth(1) {
            let rest = rest.trim();
            let expr = if rest.starts_with("(ZExt ") || rest.starts_with("(SExt ") {
                rest[6..rest.len()-1].trim()
                    .splitn(2, char::is_whitespace).nth(1).unwrap_or("").trim()
            } else { rest };
            if !expr.is_empty() { return parse_klee_bool(ctx, vars, labels, expr); }
        }
    }
    if s.starts_with("(Eq false ") {
        return parse_klee_bool(ctx, vars, labels, s["(Eq false ".len()..s.len()-1].trim())
            .map(|b| b.not());
    }
    if s.starts_with("(Eq true ") {
        return parse_klee_bool(ctx, vars, labels, s["(Eq true ".len()..s.len()-1].trim());
    }

    macro_rules! cmp {
        ($pfx:expr, $op:ident) => {
            if s.starts_with($pfx) {
                let args = s[$pfx.len()..s.len()-1].trim();
                if let Some((l, r)) = split_two(args) {
                    if let (Some(lv), Some(rv)) = (
                        parse_klee_int(ctx, vars, labels, l),
                        parse_klee_int(ctx, vars, labels, r),
                    ) { return Some(lv.$op(&rv)); }
                }
            }
        };
    }
    cmp!("(Slt ", lt); cmp!("(Sle ", le); cmp!("(Sgt ", gt); cmp!("(Sge ", ge);
    cmp!("(Ult ", lt); cmp!("(Ule ", le); cmp!("(Ugt ", gt); cmp!("(Uge ", ge);

    if s.starts_with("(Eq ") {
        let args = s["(Eq ".len()..s.len()-1].trim();
        if let Some((l, r)) = split_two(args) {
            if let (Some(lv), Some(rv)) = (
                parse_klee_int(ctx, vars, labels, l),
                parse_klee_int(ctx, vars, labels, r),
            ) { return Some(lv._eq(&rv)); }
        }
    }
    if s.starts_with("(Not ") {
        return parse_klee_bool(ctx, vars, labels, s["(Not ".len()..s.len()-1].trim())
            .map(|b| b.not());
    }
    if s.starts_with("(And ") {
        let args = s["(And ".len()..s.len()-1].trim();
        if let Some((l, r)) = split_two(args) {
            if let (Some(lv), Some(rv)) = (
                parse_klee_bool(ctx, vars, labels, l),
                parse_klee_bool(ctx, vars, labels, r),
            ) { return Some(Bool::and(ctx, &[&lv, &rv])); }
        }
    }
    if s.starts_with("(Or ") {
        let args = s["(Or ".len()..s.len()-1].trim();
        if let Some((l, r)) = split_two(args) {
            if let (Some(lv), Some(rv)) = (
                parse_klee_bool(ctx, vars, labels, l),
                parse_klee_bool(ctx, vars, labels, r),
            ) { return Some(Bool::or(ctx, &[&lv, &rv])); }
        }
    }
    None
}

fn parse_klee_int<'ctx>(
    ctx:    &'ctx Context,
    vars:   &HashMap<String, Int<'ctx>>,
    labels: &HashMap<String, String>,
    s:      &str,
) -> Option<Int<'ctx>> {
    let s = s.trim();
    if s.is_empty() { return None; }
    if let Some(pos) = find_label_colon(s) {
        return parse_klee_int(ctx, vars, labels, &s[pos + 1..]);
    }
    if is_bare_label(s) {
        if let Some(vname) = labels.get(s) { return vars.get(vname).cloned(); }
        if let Some(v)     = vars.get(s)   { return Some(v.clone()); }
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
        return n.parse::<i64>().ok()
            .or_else(|| n.parse::<u64>().ok().map(|v| v as i64))
            .map(|v| Int::from_i64(ctx, v));
    }
    macro_rules! arith {
        ($pfx:expr, $fn:path) => {
            if s.starts_with($pfx) {
                let args = s[$pfx.len()..s.len()-1].trim();
                if let Some((l, r)) = split_two(args) {
                    if let (Some(lv), Some(rv)) = (
                        parse_klee_int(ctx, vars, labels, l),
                        parse_klee_int(ctx, vars, labels, r),
                    ) { return Some($fn(ctx, &[&lv, &rv])); }
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
                    if let (Some(lv), Some(rv)) = (
                        parse_klee_int(ctx, vars, labels, l),
                        parse_klee_int(ctx, vars, labels, r),
                    ) { return Some(lv.$m(&rv)); }
                }
            }
        };
    }
    divrem!("(SDiv ", div); divrem!("(SRem ", rem);
    divrem!("(UDiv ", div); divrem!("(URem ", rem);
    if s.starts_with("(Select ") {
        let inner = s["(Select ".len()..s.len()-1].trim();
        if let Some((cs, rest)) = split_two(inner) {
            if let Some((ts, fs)) = split_two(rest) {
                if let (Some(c), Some(tv), Some(fv)) = (
                    parse_klee_bool(ctx, vars, labels, cs),
                    parse_klee_int(ctx, vars, labels, ts),
                    parse_klee_int(ctx, vars, labels, fs),
                ) { return Some(c.ite(&tv, &fv)); }
            }
        }
    }
    for pfx in &["(SExt ", "(ZExt ", "(Trunc "] {
        if s.starts_with(pfx) {
            let inner = s[pfx.len()..s.len()-1].trim();
            let body = inner.splitn(2, char::is_whitespace).nth(1)
                .unwrap_or(inner).trim();
            return parse_klee_int(ctx, vars, labels, body);
        }
    }
    if s.starts_with("(Extract ") {
        let inner = s["(Extract ".len()..s.len()-1].trim();
        if let Some(body) = inner.splitn(2, char::is_whitespace).nth(1) {
            return parse_klee_int(ctx, vars, labels, body.trim());
        }
    }
    s.parse::<i64>().ok()
        .or_else(|| s.parse::<u64>().ok().map(|v| v as i64))
        .map(|v| Int::from_i64(ctx, v))
}

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
    !s.is_empty()
        && !s.starts_with('(')
        && s.chars().all(|c| c.is_alphanumeric() || c == '_')
        && s.parse::<i64>().is_err()
        && s.parse::<u64>().is_err()
}

fn split_two(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    if s.is_empty() { return None; }
    let end = if s.starts_with('(') {
        let mut depth = 0usize;
        let mut ep    = 0usize;
        for (i, c) in s.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => { depth -= 1; if depth == 0 { ep = i; break; } }
                _   => {}
            }
        }
        ep + 1
    } else {
        s.find(char::is_whitespace)?
    };
    let first  = s[..end].trim();
    let second = s[end..].trim();
    if first.is_empty() || second.is_empty() { None } else { Some((first, second)) }
}

// ── Counterexample formatting ─────────────────────────────────────────────────

impl Counterexample {
    pub fn format_inputs(&self) -> String {
        self.input_strings.iter()
            .map(|(n, v)| format!("{}={}", n, v))
            .collect::<Vec<_>>()
            .join(", ")
    }
}