// src/normalizer/mod.rs
// ═══════════════════════════════════════════════════════
// Module 3: IR Normalization
// ═══════════════════════════════════════════════════════
//
// !! CRITICAL DESIGN NOTE !!
// The bitcode files passed to KLEE must NOT be run through opt.
// LLVM's mem2reg, dce, and instcombine passes can eliminate
// klee_make_symbolic / klee_assume calls or merge branches,
// causing KLEE to see only 1 path instead of 3.
//
// This module therefore:
//   1. Makes a DIRECT copy of the compiler output for KLEE use.
//   2. Produces a SEPARATE normalized copy (for the .ll display only).
// ═══════════════════════════════════════════════════════

use crate::types::AnalysisConfig;
use crate::compiler::IrFiles;
use anyhow::Result;
use std::process::Command;
use std::fs;

#[derive(Debug, Clone)]
pub struct NormalizedFiles {
    /// These are passed to the instrumentor → KLEE.
    /// They are DIRECT COPIES of the compiler output — no opt passes.
    pub c_normalized_path:    String,
    pub rust_normalized_path: String,
}

pub fn normalize(config: &AnalysisConfig, ir_files: &IrFiles) -> Result<NormalizedFiles> {
    let c_norm = format!(
        "/tmp/equivalence_checker/{}_c_normalized.bc",
        config.function_name
    );
    let rust_norm = format!(
        "/tmp/equivalence_checker/{}_rust_normalized.bc",
        config.function_name
    );

    // ── For KLEE: direct copy, no opt ─────────────────
    println!("  Copying C IR (no opt — preserves KLEE symbolic semantics)...");
    fs::copy(&ir_files.c_ir_path, &c_norm)?;
    println!("    → {}", c_norm);

    println!("  Copying Rust IR (no opt — preserves KLEE symbolic semantics)...");
    fs::copy(&ir_files.rust_ir_path, &rust_norm)?;
    println!("    → {}", rust_norm);

    // ── For display: produce optimized .ll for human reading ──
    let ll_dir = "output/ir";
    fs::create_dir_all(ll_dir)?;

    let c_norm_ll = format!("{}/{}_c_normalized.ll", ll_dir, config.function_name);
    let r_norm_ll = format!("{}/{}_rust_normalized.ll", ll_dir, config.function_name);

    // We run opt on a throw-away copy so the KLEE bc is never touched
    let c_opt_tmp  = format!("/tmp/equivalence_checker/{}_c_opt_display.bc", config.function_name);
    let rs_opt_tmp = format!("/tmp/equivalence_checker/{}_rs_opt_display.bc", config.function_name);

    if run_opt_passes(&ir_files.c_ir_path, &c_opt_tmp).is_ok() {
        let _ = Command::new("llvm-dis-15")
            .args([&c_opt_tmp, "-o", &c_norm_ll])
            .output();
    } else {
        // Fall back to disassembling the raw bc
        let _ = Command::new("llvm-dis-15")
            .args([&ir_files.c_ir_path, "-o", &c_norm_ll])
            .output();
    }

    if run_opt_passes(&ir_files.rust_ir_path, &rs_opt_tmp).is_ok() {
        let _ = Command::new("llvm-dis-15")
            .args([&rs_opt_tmp, "-o", &r_norm_ll])
            .output();
    } else {
        let _ = Command::new("llvm-dis-15")
            .args([&ir_files.rust_ir_path, "-o", &r_norm_ll])
            .output();
    }

    Ok(NormalizedFiles {
        c_normalized_path:    c_norm,
        rust_normalized_path: rust_norm,
    })
}

/// Run standard normalization passes.
/// Only used for generating human-readable .ll; never applied to KLEE bc.
fn run_opt_passes(input: &str, output: &str) -> Result<()> {
    // Try new pass manager first (LLVM 13+)
    let o = Command::new("opt-15")
        .args(["-passes=mem2reg,dce", input, "-o", output])
        .output()?;

    if o.status.success() {
        return Ok(());
    }

    // Fallback: legacy pass manager
    let o = Command::new("opt-15")
        .args(["-mem2reg", "-dce", input, "-o", output])
        .output()?;

    if !o.status.success() {
        return Err(anyhow::anyhow!("opt failed"));
    }
    Ok(())
}