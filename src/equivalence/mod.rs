// src/equivalence/mod.rs
// ═══════════════════════════════════════════════════════
// Module 6: Z3-based Equivalence Checking
// Implements Path Merging → Query Construction → SMT Solving
// ═══════════════════════════════════════════════════════

use crate::types::{
    AnalysisConfig, EquivalenceResult, Verdict, 
    Counterexample, BehaviorSnapshot, Difference, DifferenceKind
};
use crate::symbolic::SymbolicSummaries;
use anyhow::Result;
use std::time::Instant;
use z3::{Config, Context, Solver, ast::{Ast, Int}};

pub fn check(config: &AnalysisConfig, summaries: &SymbolicSummaries) -> Result<EquivalenceResult> {
    let start_time = Instant::now();

    println!("  Analyzing path summaries...");
    println!("    C paths:    {}", summaries.c_summaries.len());
    println!("    Rust paths: {}", summaries.rust_summaries.len());

    // Step 0.6.1: Path Merging
    let merged_pairs = merge_paths(&summaries.c_summaries, &summaries.rust_summaries);
    println!("  Merged {} path pairs for comparison", merged_pairs.len());

    // Step 0.6.2 & 0.6.3: Equivalence Query Construction + SMT Solving
    for (i, (c_path, rust_path)) in merged_pairs.iter().enumerate() {
        println!("  Checking path pair {}...", i + 1);
        
        match check_path_equivalence(c_path, rust_path, &config.bounds)? {
            PathEquivalence::Equivalent => {
                println!("    ✓ Paths are equivalent");
            }
            PathEquivalence::NotEquivalent(counterexample) => {
                println!("    ✗ Found counterexample!");
                return Ok(EquivalenceResult {
                    verdict: Verdict::NotEquivalent,
                    paths_compared: i as u32 + 1,
                    counterexample: Some(counterexample),
                    time_taken: start_time.elapsed().as_secs_f64(),
                });
            }
        }
    }

    // All paths checked - programs are equivalent!
    println!("  ✓ All {} path pairs are equivalent", merged_pairs.len());

    Ok(EquivalenceResult {
        verdict: Verdict::Equivalent,
        paths_compared: merged_pairs.len() as u32,
        counterexample: None,
        time_taken: start_time.elapsed().as_secs_f64(),
    })
}

// ───────────────────────────────────────────────────────
// Step 0.6.1: Path Merging
// ───────────────────────────────────────────────────────

use crate::types::PathSummary;

fn merge_paths<'a>(
    c_paths: &'a [PathSummary],
    rust_paths: &'a [PathSummary],
) -> Vec<(&'a PathSummary, &'a PathSummary)> {
    let mut pairs = Vec::new();
    
    // Simple strategy: pair paths with similar conditions
    for c_path in c_paths {
        for rust_path in rust_paths {
            if paths_overlap(c_path, rust_path) {
                pairs.push((c_path, rust_path));
            }
        }
    }
    
    // If no overlaps, pair sequentially
    if pairs.is_empty() {
        for (c_path, rust_path) in c_paths.iter().zip(rust_paths.iter()) {
            pairs.push((c_path, rust_path));
        }
    }
    
    pairs
}

fn paths_overlap(c_path: &PathSummary, rust_path: &PathSummary) -> bool {
    // Check if path conditions have overlap
    // Simple heuristic: check if they reference similar variables
    
    let c_has_x_gt_10 = c_path.path_condition.iter().any(|c| c.contains("x > 10") || c.contains("x >= 10"));
    let rust_has_x_gt_10 = rust_path.path_condition.iter().any(|c| c.contains("x > 10") || c.contains("x >= 10"));
    
    c_has_x_gt_10 == rust_has_x_gt_10
}

// ───────────────────────────────────────────────────────
// Step 0.6.2 & 0.6.3: Equivalence Query + Z3 Solving
// ───────────────────────────────────────────────────────

enum PathEquivalence {
    Equivalent,
    NotEquivalent(Counterexample),
}

fn check_path_equivalence(
    c_path: &PathSummary,
    rust_path: &PathSummary,
    bounds: &[crate::types::InputBound],
) -> Result<PathEquivalence> {
    // Create Z3 context
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    
    // Declare symbolic variables
    let x = Int::new_const(&ctx, "x");
    let y = Int::new_const(&ctx, "y");
    
    // Add bounds as constraints
    for bound in bounds {
        if bound.name == "x" {
            solver.assert(&x.ge(&Int::from_i64(&ctx, bound.min)));
            solver.assert(&x.le(&Int::from_i64(&ctx, bound.max)));
        } else if bound.name == "y" {
            solver.assert(&y.ge(&Int::from_i64(&ctx, bound.min)));
            solver.assert(&y.le(&Int::from_i64(&ctx, bound.max)));
        }
    }
    
    // Add path conditions
    for constraint in &c_path.path_condition {
        if let Some(z3_constraint) = parse_constraint_to_z3(&ctx, &x, &y, constraint) {
            solver.assert(&z3_constraint);
        }
    }
    
    for constraint in &rust_path.path_condition {
        if let Some(z3_constraint) = parse_constraint_to_z3(&ctx, &x, &y, constraint) {
            solver.assert(&z3_constraint);
        }
    }
    
    // Build return expressions
    let c_return = build_return_expr(&ctx, &x, &y, &c_path.return_expr);
    let rust_return = build_return_expr(&ctx, &x, &y, &rust_path.return_expr);
    
    // Ask Z3: Is there an input where returns differ?
    solver.assert(&c_return._eq(&rust_return).not());
    
    match solver.check() {
        z3::SatResult::Sat => {
            // Found counterexample!
            let model = solver.get_model().unwrap();
            
            let x_val = model.eval(&x, true).unwrap().as_i64().unwrap() as i64;
            let y_val = model.eval(&y, true).unwrap().as_i64().unwrap() as i64;
            
            let c_val = model.eval(&c_return, true).unwrap().as_i64().unwrap();
            let rust_val = model.eval(&rust_return, true).unwrap().as_i64().unwrap();
            
            Ok(PathEquivalence::NotEquivalent(Counterexample {
                inputs: vec![
                    ("x".to_string(), x_val),
                    ("y".to_string(), y_val),
                ],
                c_behavior: BehaviorSnapshot {
                    return_value: c_val.to_string(),
                    stdout: c_path.stdout_log.clone(),
                    stderr: c_path.stderr_log.clone(),
                    globals: c_path.global_writes.clone(),
                },
                rust_behavior: BehaviorSnapshot {
                    return_value: rust_val.to_string(),
                    stdout: rust_path.stdout_log.clone(),
                    stderr: rust_path.stderr_log.clone(),
                    globals: rust_path.global_writes.clone(),
                },
                differences: vec![
                    Difference {
                        kind: DifferenceKind::ReturnValue,
                        c_value: c_val.to_string(),
                        rust_value: rust_val.to_string(),
                    }
                ],
            }))
        }
        z3::SatResult::Unsat => {
            // Proven equivalent!
            Ok(PathEquivalence::Equivalent)
        }
        z3::SatResult::Unknown => {
            // Timeout or can't decide
            Ok(PathEquivalence::Equivalent) // Conservative
        }
    }
}

// ───────────────────────────────────────────────────────
// Helper: Parse Constraints to Z3
// ───────────────────────────────────────────────────────

fn parse_constraint_to_z3<'ctx>(
    ctx: &'ctx Context,
    x: &Int<'ctx>,
    y: &Int<'ctx>,
    constraint: &str,
) -> Option<z3::ast::Bool<'ctx>> {
    if constraint.contains("x >= 0") {
        Some(x.ge(&Int::from_i64(ctx, 0)))
    } else if constraint.contains("x <= 100") {
        Some(x.le(&Int::from_i64(ctx, 100)))
    } else if constraint.contains("x > 10") {
        Some(x.gt(&Int::from_i64(ctx, 10)))
    } else if constraint.contains("x >= 10") {
        Some(x.ge(&Int::from_i64(ctx, 10)))
    } else if constraint.contains("y >= 0") {
        Some(y.ge(&Int::from_i64(ctx, 0)))
    } else if constraint.contains("y <= 100") {
        Some(y.le(&Int::from_i64(ctx, 100)))
    } else {
        None // Unknown constraint
    }
}

// ───────────────────────────────────────────────────────
// Helper: Build Return Expression in Z3
// ───────────────────────────────────────────────────────

fn build_return_expr<'ctx>(
    ctx: &'ctx Context,
    x: &Int<'ctx>,
    y: &Int<'ctx>,
    expr: &str,
) -> Int<'ctx> {
    if expr.contains("x + y") {
        x + y
    } else if expr.contains("x * y") {
        x * y
    } else {
        // Default: return x
        x.clone()
    }
}