// src/validator/mod.rs
// ═══════════════════════════════════════════════════════
// Module 1: Input Validation
// Validates source files, checks function exists
// ═══════════════════════════════════════════════════════

use crate::types::{AnalysisConfig, ValidationResult, FunctionSignature, CheckerError};
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Main validation entry point
pub fn validate(config: &AnalysisConfig) -> Result<ValidationResult> {
    let mut errors = Vec::new();

    println!("  Checking C file...");
    
    // Check C file exists
    if !Path::new(&config.c_file).exists() {
        errors.push(format!("C file not found: {}", config.c_file));
        return Ok(ValidationResult {
            success: false,
            c_signature: None,
            rust_signature: None,
            errors,
        });
    }

    println!("  Checking Rust file...");
    
    // Check Rust file exists
    if !Path::new(&config.rust_file).exists() {
        errors.push(format!("Rust file not found: {}", config.rust_file));
        return Ok(ValidationResult {
            success: false,
            c_signature: None,
            rust_signature: None,
            errors,
        });
    }

    println!("  Validating C syntax...");
    
    // Validate C syntax
    if let Err(e) = validate_c_syntax(&config.c_file) {
        errors.push(format!("C syntax error: {}", e));
    }

    println!("  Validating Rust syntax...");
    
    // Validate Rust syntax
    if let Err(e) = validate_rust_syntax(&config.rust_file) {
        errors.push(format!("Rust syntax error: {}", e));
    }

    // If syntax errors, stop here
    if !errors.is_empty() {
        return Ok(ValidationResult {
            success: false,
            c_signature: None,
            rust_signature: None,
            errors,
        });
    }

    println!("  Looking for function '{}'...", config.function_name);
    
    // Check function exists in C
    let c_sig = match find_c_function(&config.c_file, &config.function_name) {
        Ok(sig) => {
            println!("    Found in C: {} with {} parameters", 
                sig.name, sig.params.len());
            Some(sig)
        }
        Err(e) => {
            errors.push(format!("C function '{}' not found: {}", 
                config.function_name, e));
            None
        }
    };

    // Check function exists in Rust
    let rust_sig = match find_rust_function(&config.rust_file, &config.function_name) {
        Ok(sig) => {
            println!("    Found in Rust: {} with {} parameters", 
                sig.name, sig.params.len());
            Some(sig)
        }
        Err(e) => {
            errors.push(format!("Rust function '{}' not found: {}", 
                config.function_name, e));
            None
        }
    };

    // Check signatures are compatible
    if let (Some(ref c), Some(ref r)) = (&c_sig, &rust_sig) {
        if c.params.len() != r.params.len() {
            errors.push(format!(
                "Parameter count mismatch: C has {}, Rust has {}",
                c.params.len(), r.params.len()
            ));
        }
    }

    Ok(ValidationResult {
        success: errors.is_empty(),
        c_signature: c_sig,
        rust_signature: rust_sig,
        errors,
    })
}

// ───────────────────────────────────────────────────────
// C FILE VALIDATION
// ───────────────────────────────────────────────────────

/// Check C syntax using clang
fn validate_c_syntax(c_file: &str) -> Result<()> {
    let output = Command::new("clang")
        .args(&["-fsyntax-only", c_file])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("{}", stderr));
    }

    Ok(())
}

/// Find function in C file
fn find_c_function(c_file: &str, func_name: &str) -> Result<FunctionSignature> {
    let content = fs::read_to_string(c_file)?;
    
    // Very simple pattern matching (not perfect but works for basic cases)
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Look for function definition like: int compute(int x, int y)
        if trimmed.contains(func_name) && trimmed.contains('(') {
            // Extract function signature
            if let Some(sig) = extract_c_signature(trimmed, func_name) {
                return Ok(sig);
            }
        }
    }

    Err(anyhow::anyhow!("Function not found"))
}

/// Extract signature from C function line
fn extract_c_signature(line: &str, func_name: &str) -> Option<FunctionSignature> {
    // Find opening paren
    let paren_start = line.find('(')?;
    let paren_end = line.find(')')?;

    // Extract parameter string
    let params_str = &line[paren_start + 1..paren_end];
    
    // Split by comma and extract parameter types
    let params: Vec<String> = params_str
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty() && *p != "void")
        .map(|p| {
            // Extract type (everything before last word)
            p.split_whitespace()
                .take_while(|&w| w != "const" && !w.starts_with('*'))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();

    // Extract return type (rough heuristic)
    let return_type = if line.contains("void") && line.find("void")? < paren_start {
        "void".to_string()
    } else if line.contains("int") && line.find("int")? < paren_start {
        "int".to_string()
    } else {
        "unknown".to_string()
    };

    Some(FunctionSignature {
        name: func_name.to_string(),
        params,
        return_type,
    })
}

// ───────────────────────────────────────────────────────
// RUST FILE VALIDATION
// ───────────────────────────────────────────────────────

/// Check Rust syntax using rustc
fn validate_rust_syntax(rust_file: &str) -> Result<()> {
    let output = Command::new("rustc")
        .args(&["--crate-type", "lib", "-Zparse-only", rust_file])
        .output()?;

    // rustc parse-only might not work, try simpler check
    if !output.status.success() {
        // Try just checking if it compiles to IR
        let output2 = Command::new("rustc")
            .args(&["--emit=llvm-ir", "--crate-type", "lib", rust_file, "-o", "/tmp/test.ll"])
            .output()?;
        
        if !output2.status.success() {
            let stderr = String::from_utf8_lossy(&output2.stderr);
            return Err(anyhow::anyhow!("{}", stderr));
        }
    }

    Ok(())
}

/// Find function in Rust file
fn find_rust_function(rust_file: &str, func_name: &str) -> Result<FunctionSignature> {
    let content = fs::read_to_string(rust_file)?;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Look for function definition: fn compute(
        if trimmed.starts_with("fn") && trimmed.contains(func_name) && trimmed.contains('(') {
            if let Some(sig) = extract_rust_signature(trimmed, func_name) {
                return Ok(sig);
            }
        }
        
        // Also check: pub fn compute(
        if trimmed.starts_with("pub fn") && trimmed.contains(func_name) && trimmed.contains('(') {
            if let Some(sig) = extract_rust_signature(trimmed, func_name) {
                return Ok(sig);
            }
        }
    }

    Err(anyhow::anyhow!("Function not found"))
}

/// Extract signature from Rust function line
fn extract_rust_signature(line: &str, func_name: &str) -> Option<FunctionSignature> {
    let paren_start = line.find('(')?;
    let paren_end = line.find(')')?;

    let params_str = &line[paren_start + 1..paren_end];
    
    // Extract parameter types
    let params: Vec<String> = params_str
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .map(|p| {
            // Extract type after colon
            if let Some(colon_pos) = p.find(':') {
                p[colon_pos + 1..].trim().to_string()
            } else {
                "unknown".to_string()
            }
        })
        .collect();

    // Extract return type
    let return_type = if line.contains("->") {
        if let Some(arrow_pos) = line.find("->") {
            let after_arrow = &line[arrow_pos + 2..];
            after_arrow
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("unknown")
                .to_string()
        } else {
            "()".to_string()
        }
    } else {
        "()".to_string()
    };

    Some(FunctionSignature {
        name: func_name.to_string(),
        params,
        return_type,
    })
}