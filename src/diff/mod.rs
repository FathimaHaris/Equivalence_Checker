// src/diff/mod.rs
// Semantic divergence detection using KLEE path constraints

use crate::types::PathSummary;

#[derive(Debug, Clone)]
pub struct SemanticDiff {
    pub c_constraint:    String,
    pub rust_constraint: String,
    pub c_line:          Option<(usize, String)>,
    pub rust_line:       Option<(usize, String)>,
    pub human_c:         String,
    pub human_rust:      String,
    pub suggestion:      String,
}

pub fn find_semantic_divergence(
    c_path:    Option<&PathSummary>,
    rust_path: Option<&PathSummary>,
    c_src:     &str,
    rust_src:  &str,
) -> Option<SemanticDiff> {
    let cp = c_path?;
    let rp = rust_path?;

    if cp.constraints == vec!["true".to_string()]
    && rp.constraints == vec!["true".to_string()] {
        return None;
    }

    // ── Find the branch constraint that differs ───────────────────────────────
    //
    // OLD (WRONG): zip() pairs constraints positionally.
    //   KLEE emits bound constraints (x >= -5, x <= 15) AND branch constraints
    //   (x >= 10). They appear in different positions for C vs Rust.
    //   zip() was pairing the BOUND constraint of C against the BRANCH
    //   constraint of Rust → showing "x > 0" instead of "x > 10".
    //
    // NEW (CORRECT): For each C constraint, find the Rust constraint that
    //   involves the SAME variable. If their normalised forms differ,
    //   that's the divergent branch.
    //
    // ─────────────────────────────────────────────────────────────────────────

    // Step 1: find constraints unique to each program (branch constraints).
    // A constraint is "shared" (a bound) if its normalised form exists in
    // both C and Rust. We want only the non-shared ones.
    let c_norms:    Vec<String> = cp.constraints.iter().map(|c| normalise_smt(c)).collect();
    let rust_norms: Vec<String> = rp.constraints.iter().map(|c| normalise_smt(c)).collect();

    let c_branch: Vec<&String> = cp.constraints.iter()
        .filter(|c| !rust_norms.contains(&normalise_smt(c)))
        .collect();

    let rust_branch: Vec<&String> = rp.constraints.iter()
        .filter(|c| !c_norms.contains(&normalise_smt(c)))
        .collect();

    // Step 2: for each C branch constraint, find the Rust branch constraint
    // that involves the SAME variables (same variable name in ReadLSB).
    for cc in &c_branch {
        let vars_c = extract_variables(cc);
        if vars_c.is_empty() { continue; }

        // Find the Rust constraint about the same variable
        let rc_match = rust_branch.iter().find(|rc| {
            let vars_r = extract_variables(rc);
            vars_c.iter().any(|v| vars_r.contains(v))
        });

        // Also try all Rust constraints if not found in branch-only set
        let rc = rc_match.copied().or_else(|| {
            rp.constraints.iter().find(|rc| {
                let vars_r = extract_variables(rc);
                vars_c.iter().any(|v| vars_r.contains(v))
                    && normalise_smt(rc) != normalise_smt(cc)
            })
        });

        if let Some(rc) = rc {
            if normalise_smt(cc) != normalise_smt(rc) {
                let human_c    = smt_to_human(cc);
                let human_rust = smt_to_human(rc);

                // Only report if the human-readable forms actually differ
                if human_c == human_rust { continue; }

                let c_line     = find_condition_in_source(c_src,    &human_c);
                let rust_line  = find_condition_in_source(rust_src, &human_rust);
                let suggestion = build_suggestion_text(
                    &human_c, &human_rust,
                    c_line.as_ref(), rust_line.as_ref(),
                );
                return Some(SemanticDiff {
                    c_constraint:    cc.to_string(),
                    rust_constraint: rc.clone(),
                    c_line,
                    rust_line,
                    human_c,
                    human_rust,
                    suggestion,
                });
            }
        }
    }

    // Step 3: fallback — return expression differs
    if cp.return_expr != rp.return_expr {
        let human_c    = cp.return_expr.as_deref()
            .map(smt_to_human).unwrap_or_else(|| "?".into());
        let human_rust = rp.return_expr.as_deref()
            .map(smt_to_human).unwrap_or_else(|| "?".into());
        let suggestion = format!(
            "Return expressions differ:\n  C    computes: {}\n  Rust computes: {}\n\nCheck arithmetic in the return statement.",
            human_c, human_rust
        );
        return Some(SemanticDiff {
            c_constraint:    human_c.clone(),
            rust_constraint: human_rust.clone(),
            c_line:          None,
            rust_line:       None,
            human_c,
            human_rust,
            suggestion,
        });
    }

    None
}

// ── Variable extraction ───────────────────────────────────────────────────────

/// Extract variable names from a KLEE constraint.
/// "(Slt (ReadLSB w32 0 x) (w32 10))"  →  ["x"]
fn extract_variables(constraint: &str) -> Vec<String> {
    let mut vars = Vec::new();
    let mut remaining = constraint;
    while let Some(pos) = remaining.find("ReadLSB") {
        let after = &remaining[pos..];
        // ReadLSB w32 OFFSET VARNAME)
        let parts: Vec<&str> = after.split_whitespace().collect();
        if parts.len() >= 4 {
            let name = parts[3].trim_end_matches(')').trim_end_matches(',');
            if !name.is_empty()
                && name.chars().next().map(|c| c.is_alphabetic() || c == '_').unwrap_or(false)
            {
                vars.push(name.to_string());
            }
        }
        remaining = &remaining[pos + 7..];
    }
    vars.dedup();
    vars
}

// ── smt_to_human ─────────────────────────────────────────────────────────────

pub fn smt_to_human(s: &str) -> String {
    let s = strip_array_prefix(s.trim());
    let s = s.trim();

    // (Eq false INNER) — logical negation
    if s.starts_with("(Eq false ") {
        let inner = s["(Eq false ".len()..].trim_end();
        let inner = if inner.ends_with(')') { &inner[..inner.len()-1] } else { inner };
        return negate_human(&smt_to_human(inner.trim()));
    }

    // (Eq true INNER) — identity
    if s.starts_with("(Eq true ") {
        let inner = s["(Eq true ".len()..].trim_end();
        let inner = if inner.ends_with(')') { &inner[..inner.len()-1] } else { inner };
        return smt_to_human(inner.trim());
    }

    // Comparison operators
    let cmp_ops: &[(&str, &str)] = &[
        ("(Sgt ", ">"),  ("(Sge ", ">="),
        ("(Slt ", "<"),  ("(Sle ", "<="),
        ("(Ugt ", ">"),  ("(Uge ", ">="),
        ("(Ult ", "<"),  ("(Ule ", "<="),
        ("(Eq ",  "=="),
    ];
    for &(prefix, op) in cmp_ops {
        if let Some(rest) = s.strip_prefix(prefix) {
            let rest = rest.trim_end();
            let rest = if rest.ends_with(')') { &rest[..rest.len()-1] } else { rest };
            if let Some((l, r)) = split_two_smt(rest) {
                let lh = smt_to_human(l);
                let rh = smt_to_human(r);
                // Put variable on the left: "(Sle (w32 10) x)" → "x >= 10"
                if is_constant(l) && !is_constant(r) {
                    return format!("{} {} {}", rh, flip_op(op), lh);
                }
                return format!("{} {} {}", lh, op, rh);
            }
        }
    }

    // (ReadLSB w32 OFFSET VARNAME)
    if s.starts_with("(ReadLSB ") {
        let inner = s["(ReadLSB ".len()..].trim_end_matches(')');
        if let Some(name) = inner.split_whitespace().last() {
            return name.to_string();
        }
    }

    // (w32 N) or (w64 N)
    if (s.starts_with("(w32 ") || s.starts_with("(w64 ")) && s.ends_with(')') {
        return s[5..s.len()-1].trim().to_string();
    }

    // Arithmetic
    for &(prefix, op) in &[("(Add ", "+"), ("(Sub ", "-"), ("(Mul ", "*")] {
        if let Some(rest) = s.strip_prefix(prefix) {
            let rest = rest.trim_end_matches(')');
            if let Some((l, r)) = split_two_smt(rest) {
                return format!("({} {} {})", smt_to_human(l), op, smt_to_human(r));
            }
        }
    }

    if s.parse::<i64>().is_ok() { return s.to_string(); }

    // Fallback
    s.replace("(w32 ", "")
     .replace("(w64 ", "")
     .replace(')', "")
     .split_whitespace()
     .collect::<Vec<_>>()
     .join(" ")
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn strip_array_prefix(s: &str) -> &str {
    if let Some(colon) = s.find(':') {
        let prefix = &s[..colon];
        if !prefix.contains(' ') && !prefix.contains('(') && !prefix.contains(')') {
            return s[colon + 1..].trim();
        }
    }
    s
}

fn negate_human(s: &str) -> String {
    let ops = [(">=", "<"), ("<=", ">"), (">", "<="), ("<", ">="),
               ("==", "!="), ("!=", "==")];
    for (op, neg) in &ops {
        if let Some(pos) = find_op_pos(s, op) {
            let left  = s[..pos].trim();
            let right = s[pos + op.len()..].trim();
            return format!("{} {} {}", left, neg, right);
        }
    }
    format!("NOT ({})", s)
}

fn find_op_pos(s: &str, op: &str) -> Option<usize> {
    let mut depth = 0i32;
    let bytes = s.as_bytes();
    for i in 0..s.len() {
        match bytes[i] { b'(' => depth += 1, b')' => depth -= 1, _ => {} }
        if depth == 0 && s[i..].starts_with(op) {
            let before_ok = i == 0 || bytes[i-1] == b' ';
            let after_ok  = s[i + op.len()..].starts_with(' ');
            if before_ok && after_ok { return Some(i); }
        }
    }
    None
}

fn is_constant(s: &str) -> bool {
    let s = s.trim();
    s.starts_with("(w32 ") || s.starts_with("(w64 ") || s.parse::<i64>().is_ok()
}

fn flip_op(op: &str) -> &str {
    match op { ">" => "<", "<" => ">", ">=" => "<=", "<=" => ">=", _ => op }
}

fn normalise_smt(s: &str) -> String {
    let s = strip_array_prefix(s.trim());
    s.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
     .replace("sle","le").replace("slt","lt")
     .replace("sge","ge").replace("sgt","gt")
     .replace("ule","le").replace("ult","lt")
     .replace("uge","ge").replace("ugt","gt")
}

fn split_two_smt(s: &str) -> Option<(&str, &str)> {
    let s = s.trim();
    let end = if s.starts_with('(') {
        let mut depth = 0usize; let mut pos = 0usize;
        for (i, c) in s.char_indices() {
            match c { '(' => depth += 1, ')' => { depth -= 1; if depth == 0 { pos = i; break; } } _ => {} }
        }
        pos + 1
    } else { s.find(char::is_whitespace)? };
    let first = s[..end].trim(); let second = s[end..].trim();
    if first.is_empty() || second.is_empty() { None } else { Some((first, second)) }
}

fn find_condition_in_source(src: &str, human_condition: &str) -> Option<(usize, String)> {
    let tokens: Vec<&str> = human_condition
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .collect();
    if tokens.is_empty() { return None; }
    for (i, line) in src.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("if") || trimmed.starts_with("while")
        || trimmed.starts_with("} else if") {
            if tokens.iter().all(|t| line.contains(t)) {
                return Some((i + 1, trimmed.to_string()));
            }
        }
    }
    None
}

fn extract_condition(line: &str) -> Option<String> {
    let line = line.trim();
    let after = if let Some(r) = line.strip_prefix("if (")            { r }
                else if let Some(r) = line.strip_prefix("if(")        { r }
                else if let Some(r) = line.strip_prefix("if ")         { r }
                else if let Some(r) = line.strip_prefix("} else if (") { r }
                else if let Some(r) = line.strip_prefix("} else if ")  { r }
                else { return None; };
    Some(after.trim_end_matches('{').trim_end().trim_end_matches(')').trim_start_matches('(').trim().to_string())
}

fn build_suggestion_text(
    human_c:    &str,
    human_rust: &str,
    c_line:     Option<&(usize, String)>,
    rust_line:  Option<&(usize, String)>,
) -> String {
    let mut out = String::from("C is the source of truth. The branch condition differs:\n\n");
    match c_line {
        Some((n, l)) => out.push_str(&format!("  C    line {}: {}\n", n, l)),
        None         => out.push_str(&format!("  C    condition: {}\n", human_c)),
    }
    match rust_line {
        Some((n, l)) => out.push_str(&format!("  Rust line {}: {}\n", n, l)),
        None         => out.push_str(&format!("  Rust condition: {}\n", human_rust)),
    }
    out.push('\n');
    if let Some((rn, rl)) = rust_line {
        if let Some((_, cl)) = c_line {
            if let (Some(cc), Some(rc)) = (extract_condition(cl), extract_condition(rl)) {
                let fixed = rl.replacen(&rc, &cc, 1);
                out.push_str(&format!("Suggested fix on Rust line {}:\n  - {}\n  + {}\n", rn, rl, fixed));
                return out;
            }
        }
        out.push_str(&format!(
            "Suggested fix on Rust line {}:\n  Change condition from `{}` to match C: `{}`\n",
            rn, human_rust, human_c
        ));
    } else {
        out.push_str(&format!("Change Rust condition from `{}` to `{}`\n", human_rust, human_c));
    }
    out
}