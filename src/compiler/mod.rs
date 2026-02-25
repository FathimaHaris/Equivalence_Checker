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


/// Generate KLEE harness for C function
fn generate_c_harness(
    c_file: &str,
    function_name: &str,
    bounds: &[crate::types::InputBound],
) -> Result<String> {
    println!("    Generating C harness with KLEE directives...");
    
    // Read original C file
    let content = fs::read_to_string(c_file)?;
    
    // Build harness
    let mut harness = String::new();
    harness.push_str("#include <klee/klee.h>\n\n");
    harness.push_str("// Original function\n");
    harness.push_str(&content);
    harness.push_str("\n\n// Auto-generated KLEE harness\n");
    harness.push_str("int main() {\n");
    
    // Declare variables
    for bound in bounds {
        harness.push_str(&format!("    int {};\n", bound.name));
    }
    harness.push_str("\n");
    
    // Make symbolic
    for bound in bounds {
        harness.push_str(&format!(
            "    klee_make_symbolic(&{}, sizeof({}), \"{}\");\n",
            bound.name, bound.name, bound.name
        ));
    }
    harness.push_str("\n");
    
    // Add bounds
    for bound in bounds {
        harness.push_str(&format!(
            "    klee_assume({} >= {} && {} <= {});\n",
            bound.name, bound.min, bound.name, bound.max
        ));
    }
    harness.push_str("\n");
    
    // Call function
    let args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    harness.push_str(&format!(
        "    int result = {}({});\n",
        function_name,
        args.join(", ")
    ));
    harness.push_str("    return result;\n");
    harness.push_str("}\n");
    
    // Write harness
    let harness_path = format!("/tmp/equivalence_checker/{}_c_harness.c", function_name);
    fs::write(&harness_path, harness)?;
    
    Ok(harness_path)
}

/// Generate KLEE harness for Rust function
fn generate_rust_harness(
    rust_file: &str,
    function_name: &str,
    bounds: &[crate::types::InputBound],
) -> Result<String> {
    println!("    Generating Rust harness with KLEE FFI...");
    
    let content = fs::read_to_string(rust_file)?;
    
    let mut harness = String::new();
    
    // Add KLEE FFI declarations (using std instead of core)
    harness.push_str("use std::os::raw::c_void;\n\n");
    harness.push_str("extern \"C\" {\n");
    harness.push_str("    fn klee_make_symbolic(addr: *mut c_void, nbytes: usize, name: *const u8);\n");
    harness.push_str("    fn klee_assume(cond: i32);\n");
    harness.push_str("}\n\n");
    
    // Original function
    harness.push_str(&content);
    harness.push_str("\n\n");
    
    // Main harness
    harness.push_str("#[no_mangle]\n");
    harness.push_str("pub extern \"C\" fn klee_harness() -> i32 {\n");  
    
    // Declare variables as mutable
    for bound in bounds {
        harness.push_str(&format!("    let mut {}: i32 = 0;\n", bound.name));
    }
    harness.push_str("\n");
    
    // Make symbolic
    harness.push_str("    unsafe {\n");
    for bound in bounds {
        harness.push_str(&format!(
            "        klee_make_symbolic(\n            \
&mut {} as *mut i32 as *mut c_void,\n            \
std::mem::size_of::<i32>(),\n            \
b\"{}\\0\".as_ptr()\n        \
);\n",
            bound.name, bound.name
        ));
    }
    
    // Add assumptions for bounds
    for bound in bounds {
        harness.push_str(&format!(
            "        klee_assume(({} >= {} && {} <= {}) as i32);\n",
            bound.name, bound.min, bound.name, bound.max
        ));
    }
    harness.push_str("    }\n\n");
    
    // Call function
    let args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    harness.push_str(&format!(
        "    {}({})\n",
        function_name,
        args.join(", ")
    ));
    harness.push_str("}\n");
    
    let harness_path = format!("/tmp/equivalence_checker/{}_rust_harness.rs", function_name);
    fs::write(&harness_path, harness)?;
    
    Ok(harness_path)
}

/// Main compilation entry point
// pub fn compile(config: &AnalysisConfig) -> Result<IrFiles> {
//     // Create output directory if it doesn't exist
//     fs::create_dir_all("/tmp/equivalence_checker")?;

//     // Generate output file paths
//     let c_ir_path = format!("/tmp/equivalence_checker/{}_c.bc", config.function_name);
//     let rust_ir_path = format!("/tmp/equivalence_checker/{}_rust.bc", config.function_name);

//     // Step 1: Compile C to LLVM IR
//     println!("  Compiling C to LLVM IR...");
//     compile_c_to_ir(&config.c_file, &c_ir_path)?;
    
//     // Verify C IR was created
//     if !Path::new(&c_ir_path).exists() {
//         return Err(CheckerError::CompilationError(
//             "C compilation failed - no output file generated".into()
//         ).into());
//     }
    
//     println!("    → Generated: {}", c_ir_path);

//     // Step 2: Compile Rust to LLVM IR
//     println!("  Compiling Rust to LLVM IR...");
//     compile_rust_to_ir(&config.rust_file, &rust_ir_path)?;
    
//     // Verify Rust IR was created
//     if !Path::new(&rust_ir_path).exists() {
//         return Err(CheckerError::CompilationError(
//             "Rust compilation failed - no output file generated".into()
//         ).into());
//     }
    
//     println!("    → Generated: {}", rust_ir_path);

//     // Step 3: Basic file size check (instead of llvm-dis verification)
//     println!("  Verifying IR files...");
//     verify_ir_basic(&c_ir_path)?;
//     verify_ir_basic(&rust_ir_path)?;
//     println!("    → IR files created successfully");

//     Ok(IrFiles {
//         c_ir_path,
//         rust_ir_path,
//     })
// }


pub fn compile(config: &AnalysisConfig) -> Result<IrFiles> {
    // Create output directory if it doesn't exist
    fs::create_dir_all("/tmp/equivalence_checker")?;

    // Generate KLEE harnesses
    println!("  Generating KLEE harnesses...");
    let c_harness = generate_c_harness(&config.c_file, &config.function_name, &config.bounds)?;
    let rust_harness = generate_rust_harness(&config.rust_file, &config.function_name, &config.bounds)?;

    // Generate output file paths
    let c_ir_path = format!("/tmp/equivalence_checker/{}_c.bc", config.function_name);
    let rust_ir_path = format!("/tmp/equivalence_checker/{}_rust.bc", config.function_name);

    // Step 1: Compile C harness to LLVM IR
    println!("  Compiling C harness to LLVM IR...");
    compile_c_to_ir(&c_harness, &c_ir_path)?;
    
    // Verify C IR was created
    if !Path::new(&c_ir_path).exists() {
        return Err(CheckerError::CompilationError(
            "C compilation failed - no output file generated".into()
        ).into());
    }
    
    println!("    → Generated: {}", c_ir_path);

    // Step 2: Compile Rust harness to LLVM IR
    println!("  Compiling Rust harness to LLVM IR...");
    compile_rust_to_ir(&rust_harness, &rust_ir_path)?;
    
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
    let output_status = Command::new("clang-15")
        .arg("-emit-llvm")
        .arg("-c")
        .arg("-O0")
        .arg("-fno-stack-protector")
        .arg("-fno-inline")
        .arg("-I/home/fathima/klee/include")
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
    let output_status = Command::new("rustup")
        .args(["run", "1.69.0", "rustc"])
        .arg("--emit=llvm-bc")
        .arg("-C").arg("opt-level=0")
        .arg("-C").arg("inline-threshold=0")
        .arg("-C").arg("debuginfo=0")
        .arg("-C").arg("overflow-checks=off") 
        .arg("--crate-type=lib")
        .arg("-o").arg(output)
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