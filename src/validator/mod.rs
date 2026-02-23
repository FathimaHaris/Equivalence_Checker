// src/validator/mod.rs
// ═══════════════════════════════════════════════════════
// Module 1: Input Validation
// Checks file existence, syntax, and function signatures
// ═══════════════════════════════════════════════════════

use crate::types::{AnalysisConfig, ValidationResult, FunctionSignature, CheckerError};
use anyhow::Result;
use std::process::Command;
use std::path::Path;

/// Main validation entry point
pub fn validate(config: &AnalysisConfig) -> Result<ValidationResult> {
    let mut errors = Vec::new();

    // Step 1: Check file existence
    println!("  Checking file existence...");
    if !Path::new(&config.c_file).exists() {
        errors.push(format!("C file not found: {}", config.c_file));
    }
    if !Path::new(&config.rust_file).exists() {
        errors.push(format!("Rust file not found: {}", config.rust_file));
    }

    // If files don't exist, stop here
    if !errors.is_empty() {
        return Ok(ValidationResult {
            success: false,
            c_signature: None,
            rust_signature: None,
            errors,
        });
    }

    // Step 2: Check C syntax
    println!("  Checking C syntax...");
    match check_c_syntax(&config.c_file) {
        Ok(_) => {},
        Err(e) => errors.push(format!("C syntax error: {}", e)),
    }

    // Step 3: Check Rust syntax  
    println!("  Checking Rust syntax...");
    match check_rust_syntax(&config.rust_file) {
        Ok(_) => {},
        Err(e) => errors.push(format!("Rust syntax error: {}", e)),
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

    // Step 4: Find function in C file
    println!("  Looking for function '{}' in C...", config.function_name);
    let c_sig = match find_c_function(&config.c_file, &config.function_name) {
        Ok(sig) => Some(sig),
        Err(e) => {
            errors.push(format!("C function not found: {}", e));
            None
        }
    };

    // Step 5: Find function in Rust file
    println!("  Looking for function '{}' in Rust...", config.function_name);
    let rust_sig = match find_rust_function(&config.rust_file, &config.function_name) {
        Ok(sig) => Some(sig),
        Err(e) => {
            errors.push(format!("Rust function not found: {}", e));
            None
        }
    };

    // Step 6: Compare signatures if both found
    if let (Some(ref c), Some(ref r)) = (&c_sig, &rust_sig) {
        println!("  Comparing function signatures...");
        
        // Check parameter count
        if c.params.len() != r.params.len() {
            errors.push(format!(
                "Parameter count mismatch: C has {}, Rust has {}",
                c.params.len(), r.params.len()
            ));
        }

        // Check return types are compatible
        if !are_types_compatible(&c.return_type, &r.return_type) {
            errors.push(format!(
                "Return type mismatch: C returns '{}', Rust returns '{}'",
                c.return_type, r.return_type
            ));
        }
    }

    // Build result
    Ok(ValidationResult {
        success: errors.is_empty(),
        c_signature: c_sig,
        rust_signature: rust_sig,
        errors,
    })
}

// C VALIDATION HELPERS

fn check_c_syntax(c_file: &str) -> Result<()> {
    let output = Command::new("clang")
        .arg("-fsyntax-only")
        .arg(c_file)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckerError::ValidationError(stderr.to_string()).into());
    }
    Ok(())
}

fn find_c_function(c_file: &str, func_name: &str) -> Result<FunctionSignature> {
    let content = std::fs::read_to_string(c_file)?;
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("//") || line.is_empty() {
            continue;
        }
        if line.contains(func_name) && line.contains('(') {
            let sig = extract_c_signature(line, func_name)?;
            return Ok(sig);
        }
    }
    
    Err(CheckerError::ValidationError(
        format!("Function '{}' not found", func_name)
    ).into())
}

fn extract_c_signature(line: &str, func_name: &str) -> Result<FunctionSignature> {
    let func_pos = line.find(func_name).unwrap();
    let before = &line[..func_pos].trim();
    let return_type = before.split_whitespace().last().unwrap_or("void").to_string();
    
    let pstart = line.find('(').unwrap();
    let pend = line.find(')').unwrap();
    let params_str = line[pstart + 1..pend].trim();
    
    let params = if params_str.is_empty() || params_str == "void" {
        vec![]
    } else {
        params_str.split(',').map(|p| p.trim().to_string()).collect()
    };
    
    Ok(FunctionSignature {
        name: func_name.to_string(),
        params,
        return_type,
    })
}

// RUST VALIDATION HELPERS

    
    
fn check_rust_syntax(rust_file: &str) -> Result<()> {
    // Compile-check only (no linking), stable-compatible
    // We emit metadata to avoid producing a full binary.
    let output = Command::new("rustc")
        .arg("--crate-type=lib")
        .arg("--emit=metadata")
        .arg(rust_file)
        .arg("-o")
        .arg("/tmp/rust_syntax_check.rmeta")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckerError::ValidationError(stderr.to_string()).into());
    }
    Ok(())
}

fn find_rust_function(rust_file: &str, func_name: &str) -> Result<FunctionSignature> {
    let content = std::fs::read_to_string(rust_file)?;
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("//") || line.is_empty() {
            continue;
        }
        if (line.starts_with("fn") || line.starts_with("pub fn")) && line.contains(func_name) {
            let sig = extract_rust_signature(line, func_name)?;
            return Ok(sig);
        }
    }
    
    Err(CheckerError::ValidationError(
        format!("Function '{}' not found", func_name)
    ).into())
}

fn extract_rust_signature(line: &str, func_name: &str) -> Result<FunctionSignature> {
    let pstart = line.find('(').unwrap();
    let pend = line.find(')').unwrap();
    let params_str = &line[pstart + 1..pend].trim();
    
    let params = if params_str.is_empty() {
        vec![]
    } else {
        params_str.split(',').map(|p| p.trim().to_string()).collect()
    };
    
    let return_type = if line.contains("->") {
        let arrow = line.find("->").unwrap();
        line[arrow + 2..].trim().split_whitespace().next()
            .unwrap_or("()").trim_end_matches('{').trim().to_string()
    } else {
        "()".to_string()
    };
    
    Ok(FunctionSignature {
        name: func_name.to_string(),
        params,
        return_type,
    })
}

fn are_types_compatible(c_type: &str, rust_type: &str) -> bool {
    let c = c_type.trim();
    let r = rust_type.trim();
    
    if c == r { return true; }
    
    matches!((c, r),
        ("int", "i32") | ("long", "i64") | ("short", "i16") | ("char", "i8") |
        ("unsigned int", "u32") | ("unsigned long", "u64") |
        ("float", "f32") | ("double", "f64") | ("void", "()")
    )
}
