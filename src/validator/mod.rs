// src/validator/mod.rs
// ═══════════════════════════════════════════════════════
// Module 1: Input Validation
// Checks file existence, syntax, and function signatures
// ═══════════════════════════════════════════════════════

use crate::types::{AnalysisConfig, ValidationResult, FunctionSignature, CheckerError};
use anyhow::Result;
use std::process::Command;
use std::path::Path;
use serde_json::Value;
use quote::ToTokens;
use syn::{Item, ItemFn};

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
    Ok(sig) => {
        println!("  C return type: {}", sig.return_type);
        println!("  C params: {:?}", sig.params);
        Some(sig)
    }
    Err(e) => {
        errors.push(format!("C function not found: {}", e));
        None
    }
    };

    // Step 5: Find function in Rust file
    println!("  Looking for function '{}' in Rust...", config.function_name);
    let rust_sig = match find_rust_function(&config.rust_file, &config.function_name) {
        Ok(sig) => {
            println!("  Rust return type: {}", sig.return_type);
            println!("  Rust params: {:?}", sig.params);
            Some(sig)
        }
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

// fn check_c_syntax(_c_file: &str) -> Result<()> {
//     // Skip syntax check - we'll validate during compilation
//     // This avoids KLEE header requirement at validation stage
//     Ok(())
// }



pub fn find_c_function(c_file: &str, func_name: &str) -> Result<FunctionSignature> {
    // 1) Ask clang for AST in JSON form
    let output = Command::new("clang")
        .args(["-Xclang", "-ast-dump=json", "-fsyntax-only"])
        .arg(c_file)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CheckerError::ValidationError(format!(
            "clang AST dump failed: {}",
            stderr
        ))
        .into());
    }

    // 2) Parse JSON
    let json_text = String::from_utf8_lossy(&output.stdout);
    let root: Value = serde_json::from_str(&json_text).map_err(|e| {
        CheckerError::ValidationError(format!("Failed to parse clang AST JSON: {}", e))
    })?;

    // 3) Walk AST to find matching FunctionDecl
    let func_node = find_function_decl(&root, func_name)
        .ok_or_else(|| CheckerError::ValidationError(format!(
            "Function '{}' not found in C AST",
            func_name
        )))?;

    // 4) Extract signature info
    extract_signature_from_function_decl(func_node, func_name)
    
}

fn find_function_decl<'a>(node: &'a Value, func_name: &str) -> Option<&'a Value> {
    // Node kind?
    let kind = node.get("kind")?.as_str().unwrap_or("");

    if kind == "FunctionDecl" {
        let name = node.get("name")?.as_str().unwrap_or("");
        if name == func_name {
            // Prefer a definition if possible:
            // clang AST often includes "isImplicit", "isUsed", "loc", and body in "inner"
            // If it has a CompoundStmt in inner, it's likely the definition.
            if has_compound_body(node) {
                return Some(node);
            }
            // Otherwise keep it as a candidate prototype
            return Some(node);
        }
    }

    // Recurse into children
    if let Some(inner) = node.get("inner").and_then(|v| v.as_array()) {
        for child in inner {
            if let Some(found) = find_function_decl(child, func_name) {
                return Some(found);
            }
        }
    }

    None
}

fn has_compound_body(node: &Value) -> bool {
    if let Some(inner) = node.get("inner").and_then(|v| v.as_array()) {
        inner.iter().any(|c| c.get("kind").and_then(|k| k.as_str()) == Some("CompoundStmt"))
    } else {
        false
    }
}

fn extract_signature_from_function_decl(node: &Value, func_name: &str) -> Result<FunctionSignature> {
    // Return type:
    // In clang json, return type usually appears in node["type"]["qualType"] like:
    // "int (int, int)"  OR sometimes return type via node["type"]["qualType"] parsing.
    // Another field often exists: node["returnType"]["qualType"] (depends on clang version).
    //
    // We'll support both.
    let return_type = if let Some(rt) = node.get("returnType").and_then(|v| v.get("qualType")).and_then(|v| v.as_str()) {
        rt.to_string()
    } else {
        // Fallback: parse from "type.qualType": "RET (ARGS...)"
        let qual = node
            .get("type")
            .and_then(|v| v.get("qualType"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| CheckerError::ValidationError("Missing type.qualType in FunctionDecl".into()))?;

        // Example: "unsigned int (int, int)"
        // Return type is everything before " ("
        let ret = qual.split(" (").next().unwrap_or(qual).trim();
        ret.to_string()
    };

    // Params: children with kind "ParmVarDecl"
    let mut params: Vec<String> = Vec::new();
    if let Some(inner) = node.get("inner").and_then(|v| v.as_array()) {
        for child in inner {
            if child.get("kind").and_then(|k| k.as_str()) == Some("ParmVarDecl") {
                let ptype = child
                    .get("type")
                    .and_then(|t| t.get("qualType"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");

                let pname = child.get("name").and_then(|n| n.as_str()).unwrap_or("");

                // Store "type name" like C syntax
                if pname.is_empty() {
                    params.push(ptype.to_string());
                } else {
                    params.push(format!("{} {}", ptype, pname));
                }
            }
        }
    }

   


    Ok(FunctionSignature {
        name: func_name.to_string(),
        params,
        return_type,
    })
}

// RUST VALIDATION HELPERS
fn check_rust_syntax(rust_file: &str) -> Result<()> {
    // Compile-check only (no linking), stable-compatible
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

    // Parse full Rust source into AST
    let file_ast = syn::parse_file(&content).map_err(|e| {
        CheckerError::ValidationError(format!("Rust parse failed: {}", e))
    })?;

    for item in file_ast.items {
        if let Item::Fn(item_fn) = item {
            if item_fn.sig.ident == func_name {
                return Ok(extract_rust_signature_from_itemfn(&item_fn));
            }
        }
    }

    Err(CheckerError::ValidationError(
        format!("Function '{}' not found", func_name)
    ).into())
}

fn extract_rust_signature_from_itemfn(item_fn: &ItemFn) -> FunctionSignature {
    let name = item_fn.sig.ident.to_string();

    // Params
    let mut params: Vec<String> = Vec::new();
    for input in &item_fn.sig.inputs {
        match input {
            syn::FnArg::Receiver(_) => {
                // "self" method; your tool probably doesn't support methods yet
                params.push("self".to_string());
            }
            syn::FnArg::Typed(pat_type) => {
                let pat = pat_type.pat.to_token_stream().to_string();
                let ty = pat_type.ty.to_token_stream().to_string();
                params.push(format!("{}: {}", pat, ty));
            }
        }
    }

    // Return type
    let return_type = match &item_fn.sig.output {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, ty) => ty.to_token_stream().to_string(),
    };

    FunctionSignature {
        name,
        params,
        return_type,
    }
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
