// src/instrumentor/mod.rs
// ═══════════════════════════════════════════════════════
// Module 4: IR Instrumentation
// Inserts hooks to track observable side effects
// ═══════════════════════════════════════════════════════

// use crate::types::{AnalysisConfig, CheckerError};
use crate::types::AnalysisConfig;
use crate::normalizer::NormalizedFiles;
use anyhow::Result;
// use std::process::Command;
use std::fs;

/// Paths to instrumented IR files
#[derive(Debug, Clone)]
pub struct InstrumentedFiles {
    pub c_instrumented_path:    String,
    pub rust_instrumented_path: String,
}

/// Main instrumentation entry point
pub fn instrument(config: &AnalysisConfig, normalized: &NormalizedFiles) -> Result<InstrumentedFiles> {
    // For now, we'll use a simplified approach:
    // Just copy the normalized files and mark them as "instrumented"
    // In a full implementation, you would:
    // 1. Parse the IR
    // 2. Find printf, global stores, etc.
    // 3. Insert __log_* calls before them


    println!("  Instrumentation disabled (return-only mode). Copying normalized IR...");

    
    let c_inst = format!("/tmp/equivalence_checker/{}_c_instrumented.bc", config.function_name);
    let rust_inst = format!("/tmp/equivalence_checker/{}_rust_instrumented.bc", config.function_name);

    println!("  Instrumenting C IR...");
    instrument_ir(&normalized.c_normalized_path, &c_inst)?;
    println!("    → Instrumented: {}", c_inst);

    println!("  Instrumenting Rust IR...");
    instrument_ir(&normalized.rust_normalized_path, &rust_inst)?;
    println!("    → Instrumented: {}", rust_inst);

    Ok(InstrumentedFiles {
        c_instrumented_path: c_inst,
        rust_instrumented_path: rust_inst,
    })
}

// ───────────────────────────────────────────────────────
// INSTRUMENTATION IMPLEMENTATION
// ───────────────────────────────────────────────────────

/// Instrument a single IR file
// fn instrument_ir(input_bc: &str, output_bc: &str) -> Result<()> {
//     // Convert to text IR for easier manipulation
//     let input_ll = input_bc.replace(".bc", "_temp.ll");
    
//     // Disassemble bitcode to text
//     let dis_result = Command::new("llvm-dis-15")
//         .arg(input_bc)
//         .arg("-o")
//         .arg(&input_ll)
//         .output();

//     // If llvm-dis fails due to version mismatch, just copy the file
//     if dis_result.is_err() || !dis_result.as_ref().unwrap().status.success() {
//         println!("    (Skipping instrumentation - copying as-is)");
//         fs::copy(input_bc, output_bc)?;
//         return Ok(());
//     }

//     // Read the IR
//     let mut content = fs::read_to_string(&input_ll)?;

//     // Apply instrumentation transformations
//     content = add_observable_declarations(content);
//     content = instrument_printf_calls(content);
//     content = instrument_global_stores(content);

//     // Write back
//     fs::write(&input_ll, content)?;

//     // Reassemble to bitcode
//     let as_result = Command::new("llvm-as-15")
//         .arg(&input_ll)
//         .arg("-o")
//         .arg(output_bc)
//         .output();

//     // If reassembly fails, just copy original
//     if as_result.is_err() || !as_result.as_ref().unwrap().status.success() {
//         println!("    (Could not reassemble - copying as-is)");
//         fs::copy(input_bc, output_bc)?;
//     }

//     // Clean up temp file
//     let _ = fs::remove_file(&input_ll);

//     Ok(())
// }



/// Return-only mode: no IR instrumentation.
/// We just forward normalized bitcode to the next stages.
fn instrument_ir(input_bc: &str, output_bc: &str) -> Result<()> {
    fs::copy(input_bc, output_bc)?;
    Ok(())
}












// // ───────────────────────────────────────────────────────
// // INSTRUMENTATION TRANSFORMATIONS
// // ───────────────────────────────────────────────────────

// /// Add declarations for our logging functions
// fn add_observable_declarations(content: String) -> String {
//     // Check if declarations already exist
//     if content.contains("@__log_stdout") {
//         return content;
//     }

//     // Find a good place to insert (after target triple)
//     let insert_point = if let Some(pos) = content.find("target triple") {
//         // Find end of that line
//         content[pos..].find('\n').map(|n| pos + n + 1).unwrap_or(content.len())
//     } else {
//         // Insert at beginning
//         0
//     };

//     let declarations = r#"
// ; ═══════════════════════════════════════════════════════
// ; Observable Logging Function Declarations
// ; ═══════════════════════════════════════════════════════

// declare void @__log_stdout(i8*)
// declare void @__log_stderr(i8*)
// declare void @__log_global_write(i8*, i32)
// declare void @__log_return(i32)

// "#;

//     let mut result = String::new();
//     result.push_str(&content[..insert_point]);
//     result.push_str(declarations);
//     result.push_str(&content[insert_point..]);
    
//     result
// }

// /// Instrument printf calls
// fn instrument_printf_calls(content: String) -> String {
//     let mut result = content;
    
//     // Simple pattern: look for "call i32 @printf"
//     // This is a simplified implementation
//     // A real implementation would properly parse the IR
    
//     // For now, just mark that we attempted instrumentation
//     result = result.replace(
//         "call i32 @printf(",
//         "; [INSTRUMENTATION POINT: printf]\n  call i32 @printf("
//     );
    
//     result
// }

// /// Instrument global variable stores
// fn instrument_global_stores(content: String) -> String {
//     let mut result = content;
    
//     // Look for stores to global variables (start with @)
//     // Pattern: store i32 %value, i32* @global_name
    
//     // Mark instrumentation points
//     result = result.replace(
//         "store i32 ",
//         "; [INSTRUMENTATION POINT: store]\n  store i32 "
//     );
    
//     result
// }

// // ───────────────────────────────────────────────────────
// // RUNTIME LIBRARY CREATION
// // ───────────────────────────────────────────────────────

// /// Create the runtime library for observable logging
// /// This would be linked with instrumented code
// #[allow(dead_code)]
// pub fn create_runtime_library() -> Result<String> {
//     let runtime_c = r#"
// // Observable Runtime Library
// // ═══════════════════════════════════════════════════════

// #include <stdio.h>
// #include <string.h>

// // Storage for observables
// static char* stdout_log[100];
// static int stdout_count = 0;

// static char* stderr_log[100];
// static int stderr_count = 0;

// static int last_return = 0;

// // Logging functions
// void __log_stdout(const char* msg) {
//     if (stdout_count < 100) {
//         stdout_log[stdout_count++] = (char*)msg;
//     }
// }

// void __log_stderr(const char* msg) {
//     if (stderr_count < 100) {
//         stderr_log[stderr_count++] = (char*)msg;
//     }
// }

// void __log_global_write(const char* name, int value) {
//     // Store global write info
// }

// void __log_return(int value) {
//     last_return = value;
// }

// // Export function
// void __observable_export(const char* filename) {
//     FILE* fp = fopen(filename, "w");
//     fprintf(fp, "{\n");
//     fprintf(fp, "  \"stdout\": [\n");
//     for (int i = 0; i < stdout_count; i++) {
//         fprintf(fp, "    \"%s\"%s\n", stdout_log[i], (i < stdout_count-1) ? "," : "");
//     }
//     fprintf(fp, "  ]\n");
//     fprintf(fp, "}\n");
//     fclose(fp);
// }
// "#;

//     // Write to file
//     let path = "/tmp/equivalence_checker/observable_runtime.c";
//     fs::write(path, runtime_c)?;
    
//     Ok(path.to_string())
// }