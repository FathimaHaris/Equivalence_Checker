// src/normalizer/mod.rs
// ═══════════════════════════════════════════════════════
// Module 3: IR Normalization
// Standardizes LLVM IR to make C and Rust comparable
// ═══════════════════════════════════════════════════════

use crate::types::{AnalysisConfig, CheckerError};
use crate::compiler::IrFiles;
use anyhow::Result;
use std::process::Command;
use std::fs;

/// Paths to normalized IR files
#[derive(Debug, Clone)]
pub struct NormalizedFiles {
    pub c_normalized_path:    String,
    pub rust_normalized_path: String,
}

/// Main normalization entry point
pub fn normalize(config: &AnalysisConfig, ir_files: &IrFiles) -> Result<NormalizedFiles> {
    // Generate output paths
    let c_norm = format!("/tmp/equivalence_checker/{}_c_normalized.bc", config.function_name);
    let rust_norm = format!("/tmp/equivalence_checker/{}_rust_normalized.bc", config.function_name);

    // Step 1: Run LLVM optimization passes on C IR
    println!("  Normalizing C IR...");
    run_normalization_passes(&ir_files.c_ir_path, &c_norm)?;
    println!("    → Normalized: {}", c_norm);

    // Step 2: Run LLVM optimization passes on Rust IR
    println!("  Normalizing Rust IR...");
    run_normalization_passes(&ir_files.rust_ir_path, &rust_norm)?;
    println!("    → Normalized: {}", rust_norm);

    // Step 3: Additional text-based normalization (optional but helpful)
    println!("  Applying additional normalizations...");
    apply_text_normalizations(&c_norm)?;
    apply_text_normalizations(&rust_norm)?;

    Ok(NormalizedFiles {
        c_normalized_path:    c_norm,
        rust_normalized_path: rust_norm,
    })
}

// ───────────────────────────────────────────────────────
// LLVM PASS-BASED NORMALIZATION
// ───────────────────────────────────────────────────────

/// Run LLVM optimization passes to normalize IR structure
/// Uses new pass manager syntax (LLVM 13+)
fn run_normalization_passes(input_bc: &str, output_bc: &str) -> Result<()> {
    let output = Command::new("opt-15")
        // Use new pass manager
        .arg("-passes=mem2reg,simplifycfg,loop-simplify,lcssa,loop-rotate,indvars,dce,instcombine")
        
        // Input file
        .arg(input_bc)
        
        // Output file
        .arg("-o")
        .arg(output_bc)
        
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // If new syntax fails, try legacy syntax
        println!("    (Trying legacy pass manager...)");
        return run_normalization_passes_legacy(input_bc, output_bc);
    }

    Ok(())
}

/// Fallback: Use legacy pass manager syntax (LLVM < 13)
fn run_normalization_passes_legacy(input_bc: &str, output_bc: &str) -> Result<()> {
    let output = Command::new("opt-15")
        .arg("-mem2reg")
        .arg("-simplifycfg")
        .arg("-loop-simplify")
        .arg("-lcssa")
        .arg("-loop-rotate")
        .arg("-dce")
        .arg("-instcombine")
        .arg(input_bc)
        .arg("-o")
        .arg(output_bc)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckerError::NormalizationError(
            format!("LLVM opt failed:\n{}", stderr)
        ).into());
    }

    Ok(())
}

// ───────────────────────────────────────────────────────
// TEXT-BASED NORMALIZATION (for consistent naming)
// ───────────────────────────────────────────────────────

/// Apply text-based normalizations by converting to .ll and back
/// This helps standardize variable and block naming
fn apply_text_normalizations(bc_path: &str) -> Result<()> {
    // Convert bitcode to text
    let ll_path = bc_path.replace(".bc", ".ll");
    
    let dis_output = Command::new("llvm-dis-15")
        .arg(bc_path)
        .arg("-o")
        .arg(&ll_path)
        .output();

    // If llvm-dis fails (version mismatch), skip text normalization
    if dis_output.is_err() || !dis_output.as_ref().unwrap().status.success() {
        println!("    (Skipping text normalization due to LLVM version)");
        return Ok(());
    }

    // Read the text IR
    let mut content = fs::read_to_string(&ll_path)?;

    // Apply text transformations
    content = normalize_block_names(content);
    content = normalize_variable_names(content);

    // Write back
    fs::write(&ll_path, content)?;

    // Convert back to bitcode
    let as_output = Command::new("llvm-as-15")
        .arg(&ll_path)
        .arg("-o")
        .arg(bc_path)
        .output();

    // If llvm-as fails, just skip (keep original)
    if as_output.is_err() || !as_output.as_ref().unwrap().status.success() {
        println!("    (Could not reassemble - keeping original)");
        return Ok(());
    }

    // Clean up .ll file
    let _ = fs::remove_file(&ll_path);

    Ok(())
}

/// Normalize basic block names to consistent patterns
fn normalize_block_names(content: String) -> String {
    let mut result = content;

    // Common patterns to standardize
    let replacements = vec![
        // Entry blocks
        ("entry:", "block_entry:"),
        
        // Loop headers
        ("for.cond:", "loop_header:"),
        ("while.cond:", "loop_header:"),
        
        // Loop bodies
        ("for.body:", "loop_body:"),
        ("while.body:", "loop_body:"),
        
        // Loop exits
        ("for.end:", "loop_exit:"),
        ("while.end:", "loop_exit:"),
        
        // Conditional branches
        ("if.then:", "branch_true:"),
        ("if.else:", "branch_false:"),
        ("if.end:", "branch_merge:"),
        
        // Return blocks
        ("return:", "block_return:"),
    ];

    for (old, new) in replacements {
        result = result.replace(old, new);
    }

    result
}

/// Normalize variable names (basic pattern matching)
fn normalize_variable_names(content: String) -> String {
    // This is a simplified version
    // A full implementation would use regex or proper IR parsing
    
    // For now, just return as-is
    // In a production tool, you'd want to:
    // - Rename %0, %1, %2 to meaningful names
    // - Standardize temporary variable naming
    // - Align parameter names
    
    content
}

// ───────────────────────────────────────────────────────
// HELPER: Display normalized IR (for debugging)
// ───────────────────────────────────────────────────────

#[allow(dead_code)]
pub fn display_normalized_ir(bc_path: &str) -> Result<()> {
    let ll_path = bc_path.replace(".bc", "_display.ll");
    
    let output = Command::new("llvm-dis")
        .arg(bc_path)
        .arg("-o")
        .arg(&ll_path)
        .output()?;

    if output.status.success() {
        let content = fs::read_to_string(&ll_path)?;
        println!("\n{}\n", content);
        let _ = fs::remove_file(&ll_path);
    }

    Ok(())
}