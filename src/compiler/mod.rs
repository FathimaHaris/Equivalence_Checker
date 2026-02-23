// src/compiler/mod.rs
// ═══════════════════════════════════════════════════════
// Module 2: LLVM IR Compilation
// Compiles C and Rust source code to LLVM IR bitcode
// ═══════════════════════════════════════════════════════

use crate::types::{AnalysisConfig, CheckerError};
use anyhow::Result;
use std::process::Command;
use std::path::Path;
use std::fs;

/// Paths to generated IR files
#[derive(Debug, Clone)]
pub struct IrFiles {
    pub c_ir_path:    String,
    pub rust_ir_path: String,
}

/// Main compilation entry point
pub fn compile(config: &AnalysisConfig) -> Result<IrFiles> {
    // Create output directory if it doesn't exist
    fs::create_dir_all("/tmp/equivalence_checker")?;

    // Generate output file paths
    let c_ir_path = format!("/tmp/equivalence_checker/{}_c.bc", config.function_name);
    let rust_ir_path = format!("/tmp/equivalence_checker/{}_rust.bc", config.function_name);

    // Step 1: Compile C to LLVM IR
    println!("  Compiling C to LLVM IR...");
    compile_c_to_ir(&config.c_file, &c_ir_path)?;
    
    // Verify C IR was created
    if !Path::new(&c_ir_path).exists() {
        return Err(CheckerError::CompilationError(
            "C compilation failed - no output file generated".into()
        ).into());
    }
    
    println!("    → Generated: {}", c_ir_path);

    // Step 2: Compile Rust to LLVM IR
    println!("  Compiling Rust to LLVM IR...");
    compile_rust_to_ir(&config.rust_file, &rust_ir_path)?;
    
    // Verify Rust IR was created
    if !Path::new(&rust_ir_path).exists() {
        return Err(CheckerError::CompilationError(
            "Rust compilation failed - no output file generated".into()
        ).into());
    }
    
    println!("    → Generated: {}", rust_ir_path);

    // Step 3: Basic file size check (instead of llvm-dis verification)
    println!("  Verifying IR files...");
    verify_ir_basic(&c_ir_path)?;
    verify_ir_basic(&rust_ir_path)?;
    println!("    → IR files created successfully");

    Ok(IrFiles {
        c_ir_path,
        rust_ir_path,
    })
}

// ───────────────────────────────────────────────────────
// C COMPILATION
// ───────────────────────────────────────────────────────

/// Compile C source to LLVM IR bitcode
fn compile_c_to_ir(c_file: &str, output: &str) -> Result<()> {
    let output_status = Command::new("clang")
        .arg("-emit-llvm")
        .arg("-c")
        .arg("-O0")
        .arg("-fno-stack-protector")
        .arg("-fno-inline")
        .arg(c_file)
        .arg("-o")
        .arg(output)
        .output()?;

    if !output_status.status.success() {
        let stderr = String::from_utf8_lossy(&output_status.stderr);
        return Err(CheckerError::CompilationError(
            format!("C compilation failed:\n{}", stderr)
        ).into());
    }

    Ok(())
}

// ───────────────────────────────────────────────────────
// RUST COMPILATION
// ───────────────────────────────────────────────────────

/// Compile Rust source to LLVM IR bitcode
fn compile_rust_to_ir(rust_file: &str, output: &str) -> Result<()> {
    let output_status = Command::new("rustc")
        .arg("--emit=llvm-bc")
        .arg("-C")
        .arg("opt-level=0")
        .arg("-C")
        .arg("inline-threshold=0")
        .arg("-C")
        .arg("debuginfo=0")
        .arg("--crate-type=lib")
        .arg("-o")
        .arg(output)
        .arg(rust_file)
        .output()?;

    if !output_status.status.success() {
        let stderr = String::from_utf8_lossy(&output_status.stderr);
        return Err(CheckerError::CompilationError(
            format!("Rust compilation failed:\n{}", stderr)
        ).into());
    }

    Ok(())
}

// ───────────────────────────────────────────────────────
// IR VERIFICATION (BASIC)
// ───────────────────────────────────────────────────────

/// Basic verification - just check file exists and has content
fn verify_ir_basic(ir_path: &str) -> Result<()> {
    let metadata = fs::metadata(ir_path)?;
    
    if metadata.len() == 0 {
        return Err(CheckerError::CompilationError(
            format!("IR file {} is empty", ir_path)
        ).into());
    }
    
    // Check if it's a bitcode file (starts with 'BC' magic bytes)
    let mut file = fs::File::open(ir_path)?;
    let mut magic = [0u8; 2];
    use std::io::Read;
    file.read_exact(&mut magic)?;
    
    if &magic != b"BC" {
        return Err(CheckerError::CompilationError(
            format!("IR file {} is not valid LLVM bitcode (missing BC magic)", ir_path)
        ).into());
    }
    
    Ok(())
}

// ───────────────────────────────────────────────────────
// OPTIONAL: HELPER TO VIEW IR IN HUMAN-READABLE FORMAT
// ───────────────────────────────────────────────────────

/// Convert bitcode to human-readable LLVM IR text (for debugging)
#[allow(dead_code)]
pub fn dump_ir_to_text(bc_path: &str, output_path: &str) -> Result<()> {
    let output = Command::new("llvm-dis")
        .arg(bc_path)
        .arg("-o")
        .arg(output_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("  Warning: Could not dump IR to text: {}", stderr);
        return Ok(()); // Don't fail, just warn
    }

    println!("  Dumped IR to: {}", output_path);
    Ok(())
}