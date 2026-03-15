// src/compiler/mod.rs
// ═══════════════════════════════════════════════════════
// Module 2: LLVM IR Compilation
//
// KEY ADDITIONS vs original:
//   - Multi-type harness generation: int, long, float, double, bool, char
//   - InputBound extended with optional type tag
//   - Harness correctly handles floats via klee_make_symbolic on float vars
//   - Runner correctly parses float args from command line
// ═══════════════════════════════════════════════════════

use crate::types::{AnalysisConfig, CheckerError};
use anyhow::Result;
use std::process::Command;
use std::path::Path;
use std::fs;

#[derive(Debug, Clone)]
pub struct IrFiles {
    pub c_ir_path:       String,
    pub rust_ir_path:    String,
    pub c_runner_bin:    String,
    pub rust_runner_bin: String,
}

// ── Type system ────────────────────────────────────────

/// The C/Rust type of one input variable.
/// Parsed from the bound spec: "x:i32:-10:10" or just "x:-10:10" (defaults to i32)
#[derive(Debug, Clone, PartialEq)]
pub enum VarType {
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    Bool,
    Char,
}

impl VarType {
    /// The C type string for declarations
    fn c_type(&self) -> &'static str {
        match self {
            VarType::I8  => "int8_t",   VarType::I16 => "int16_t",
            VarType::I32 => "int",       VarType::I64 => "long long",
            VarType::U8  => "uint8_t",  VarType::U16 => "uint16_t",
            VarType::U32 => "unsigned",  VarType::U64 => "unsigned long long",
            VarType::F32 => "float",     VarType::F64 => "double",
            VarType::Bool => "int",      VarType::Char => "char",
        }
    }

    /// The Rust type string for declarations
    fn rust_type(&self) -> &'static str {
        match self {
            VarType::I8  => "i8",  VarType::I16 => "i16",
            VarType::I32 => "i32", VarType::I64 => "i64",
            VarType::U8  => "u8",  VarType::U16 => "u16",
            VarType::U32 => "u32", VarType::U64 => "u64",
            VarType::F32 => "f32", VarType::F64 => "f64",
            VarType::Bool => "bool", VarType::Char => "char",
        }
    }

    /// sizeof in C — used for klee_make_symbolic
    fn c_sizeof(&self) -> &'static str {
        match self {
            VarType::I8  | VarType::U8  | VarType::Bool | VarType::Char => "1",
            VarType::I16 | VarType::U16 => "2",
            VarType::I32 | VarType::U32 | VarType::F32 => "4",
            VarType::I64 | VarType::U64 | VarType::F64 => "8",
        }
    }

    /// Is this a floating-point type?
    fn is_float(&self) -> bool {
        matches!(self, VarType::F32 | VarType::F64)
    }

    /// Parse from a string like "i32", "f64", "int", "double", etc.
    fn parse(s: &str) -> Option<VarType> {
        match s.trim().to_lowercase().as_str() {
            "i8"  | "int8_t"            => Some(VarType::I8),
            "i16" | "int16_t"           => Some(VarType::I16),
            "i32" | "int"               => Some(VarType::I32),
            "i64" | "long" | "long long" | "int64_t" => Some(VarType::I64),
            "u8"  | "uint8_t"           => Some(VarType::U8),
            "u16" | "uint16_t"          => Some(VarType::U16),
            "u32" | "unsigned" | "uint" | "uint32_t" => Some(VarType::U32),
            "u64" | "uint64_t"          => Some(VarType::U64),
            "f32" | "float"             => Some(VarType::F32),
            "f64" | "double"            => Some(VarType::F64),
            "bool"                      => Some(VarType::Bool),
            "char"                      => Some(VarType::Char),
            _                           => None,
        }
    }
}

/// Extended InputBound with type information
#[derive(Debug, Clone)]
struct TypedBound {
    name:     String,
    var_type: VarType,
    min:      i64,
    max:      i64,
}

/// Parse typed bounds from AnalysisConfig.
/// Format: "x:0:100" (default i32) or "x:i32:0:100" (explicit type)
fn parse_typed_bounds(config: &AnalysisConfig) -> Vec<TypedBound> {
    config.bounds.iter().map(|b| {
        // For now, default everything to i32 (can extend with type annotation in UI)
        // The type could be passed as e.g. InputBound { name, type_hint, min, max }
        TypedBound {
            name:     b.name.clone(),
            var_type: VarType::I32,  // TODO: extend InputBound with type_hint field
            min:      b.min,
            max:      b.max,
        }
    }).collect()
}

// ── Harness generation ────────────────────────────────

fn generate_c_harness(
    c_file:        &str,
    function_name: &str,
    bounds:        &[TypedBound],
) -> Result<String> {
    println!("    Generating C harness with KLEE directives...");
    let content = fs::read_to_string(c_file)?;

    let mut h = String::new();
    h.push_str("#include <klee/klee.h>\n");
    h.push_str("#include <stdint.h>\n\n");
    h.push_str("// Original function\n");
    h.push_str(&content);
    h.push_str("\n\n// Auto-generated KLEE harness\n");
    h.push_str("int main() {\n");

    // Declare variables
    for b in bounds {
        h.push_str(&format!("    {} {};\n", b.var_type.c_type(), b.name));
    }
    h.push('\n');

    // Make symbolic
    for b in bounds {
        h.push_str(&format!(
            "    klee_make_symbolic(&{name}, sizeof({name}), \"{name}\");\n",
            name = b.name
        ));
    }
    h.push('\n');

    // Apply range constraints
    for b in bounds {
        if b.var_type.is_float() {
            h.push_str(&format!(
                "    klee_assume({name} >= {min} && {name} <= {max});\n",
                name = b.name,
                min  = b.min,
                max  = b.max
            ));
        } else {
            h.push_str(&format!(
                "    klee_assume({name} >= {min} && {name} <= {max});\n",
                name = b.name,
                min  = b.min,
                max  = b.max
            ));
        }
    }
    h.push('\n');

    // Call function — cast return to volatile int to prevent elimination
    let args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    // Make the return value symbolic so KLEE includes it in the result section
    // of the .kquery file. Without this, KLEE only writes (query [constraints] false)
    // with no result expression, making symbolic comparison impossible.
    h.push_str("    int __result[1];\n");
    h.push_str("    klee_make_symbolic(__result, sizeof(__result), \"result\");\n");
    h.push_str(&format!(
        "    klee_assume(__result[0] == (int){fn_name}({args}));\n",
        fn_name = function_name,
        args    = args.join(", ")
    ));
    h.push_str("    return __result[0];\n");
    h.push_str("}\n");

    let path = format!("/tmp/equivalence_checker/{}_c_harness.c", function_name);
    fs::write(&path, h)?;
    Ok(path)
}

fn generate_rust_harness(
    rust_file:     &str,
    function_name: &str,
    bounds:        &[TypedBound],
) -> Result<String> {
    println!("    Generating Rust harness with KLEE FFI...");
    let content = fs::read_to_string(rust_file)?;

    let mut h = String::new();
    h.push_str("#![allow(unused)]\n");
    h.push_str("use std::os::raw::c_void;\n\n");
    h.push_str("extern \"C\" {\n");
    h.push_str("    fn klee_make_symbolic(addr: *mut c_void, nbytes: usize, name: *const u8);\n");
    h.push_str("    fn klee_assume(cond: i32);\n");
    h.push_str("}\n\n");
    h.push_str(&content);
    h.push_str("\n\n");
    h.push_str("#[no_mangle]\n");
    h.push_str("pub extern \"C\" fn klee_harness() -> i32 {\n");

    // Declare variables
    for b in bounds {
        let default_val = match b.var_type {
            VarType::F32 => "0.0f32".to_string(),
            VarType::F64 => "0.0f64".to_string(),
            VarType::Bool => "false".to_string(),
            VarType::Char => "'\\0'".to_string(),
            _              => "0".to_string(),
        };
        h.push_str(&format!("    let mut {}: {} = {};\n",
            b.name, b.var_type.rust_type(), default_val));
    }
    h.push('\n');

    // Make symbolic
    h.push_str("    unsafe {\n");
    for b in bounds {
        h.push_str(&format!(
            "        klee_make_symbolic(\n            &mut {name} as *mut {ty} as *mut c_void,\n            std::mem::size_of::<{ty}>(),\n            b\"{name}\\0\".as_ptr()\n        );\n",
            name = b.name,
            ty   = b.var_type.rust_type(),
        ));
    }

    // Apply constraints
    for b in bounds {
        match b.var_type {
            VarType::F32 | VarType::F64 => {
                h.push_str(&format!(
                    "        klee_assume(({name} >= {min} as {ty} && {name} <= {max} as {ty}) as i32);\n",
                    name = b.name, min = b.min, max = b.max, ty = b.var_type.rust_type()
                ));
            }
            VarType::Bool => {
                // bool is just 0 or 1
                h.push_str(&format!(
                    "        klee_assume(({name} == false || {name} == true) as i32);\n",
                    name = b.name
                ));
            }
            _ => {
                h.push_str(&format!(
                    "        klee_assume(({name} >= {min} && {name} <= {max}) as i32);\n",
                    name = b.name, min = b.min, max = b.max
                ));
            }
        }
    }
    h.push_str("    }\n\n");

    let args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    // Make the return value symbolic so KLEE tracks it in the result section.
    h.push_str("    let mut __result: i32 = 0;\n");
    h.push_str("    unsafe {\n");
    h.push_str("        klee_make_symbolic(\n");
    h.push_str("            &mut __result as *mut i32 as *mut c_void,\n");
    h.push_str("            std::mem::size_of::<i32>(),\n");
    h.push_str("            b\"result\\0\".as_ptr()\n");
    h.push_str("        );\n");
    h.push_str(&format!(
        "        klee_assume((__result == {fn_name}({args}) as i32) as i32);\n",
        fn_name = function_name,
        args    = args.join(", ")
    ));
    h.push_str("    }\n");
    h.push_str("    __result\n");
    h.push_str("}\n");
    let path = format!("/tmp/equivalence_checker/{}_rust_harness.rs", function_name);
    fs::write(&path, h)?;
    Ok(path)
}

// ── Runner generation ─────────────────────────────────

fn generate_c_runner(
    c_file:        &str,
    function_name: &str,
    bounds:        &[TypedBound],
) -> Result<String> {
    let content = fs::read_to_string(c_file)?;
    let mut s = String::new();
    s.push_str("#include <stdio.h>\n#include <stdlib.h>\n#include <stdint.h>\n\n");
    s.push_str(&content);
    s.push_str("\n\nint main(int argc, char** argv) {\n");
    s.push_str(&format!("    if (argc != {}) return 2;\n", bounds.len() + 1));
    for (i, b) in bounds.iter().enumerate() {
        let parse_fn = match b.var_type {
            VarType::F32            => format!("(float)atof(argv[{}])", i + 1),
            VarType::F64            => format!("atof(argv[{}])", i + 1),
            VarType::I64 | VarType::U64 => format!("atoll(argv[{}])", i + 1),
            VarType::Bool           => format!("(int)atoi(argv[{}])", i + 1),
            _                       => format!("atoi(argv[{}])", i + 1),
        };
        s.push_str(&format!("    {} {} = {};\n", b.var_type.c_type(), b.name, parse_fn));
    }
    let args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    s.push_str(&format!(
        "    int r = (int){}({});\n",
        function_name,
        args.join(", ")
    ));
    s.push_str("    printf(\"%d\\n\", r);\n    return 0;\n}\n");
    let path = format!("/tmp/equivalence_checker/{}_c_runner.c", function_name);
    fs::write(&path, s)?;
    Ok(path)
}

fn generate_rust_runner(
    rust_file:     &str,
    function_name: &str,
    bounds:        &[TypedBound],
) -> Result<String> {
    let content = fs::read_to_string(rust_file)?;
    let mut s = String::new();
    s.push_str("#![allow(unused)]\nuse std::env;\n\n");
    s.push_str(&content);
    s.push_str("\n\nfn main() {\n");
    s.push_str("    let args: Vec<String> = env::args().collect();\n");
    s.push_str(&format!(
        "    if args.len() != {} {{ std::process::exit(2); }}\n",
        bounds.len() + 1
    ));
    for (i, b) in bounds.iter().enumerate() {
        let parse_expr = match b.var_type {
            VarType::F32  => format!("args[{}].parse::<f32>().unwrap()", i + 1),
            VarType::F64  => format!("args[{}].parse::<f64>().unwrap()", i + 1),
            VarType::Bool => format!("args[{}].parse::<i32>().unwrap() != 0", i + 1),
            VarType::I8   => format!("args[{}].parse::<i8>().unwrap()", i + 1),
            VarType::I16  => format!("args[{}].parse::<i16>().unwrap()", i + 1),
            VarType::I32  => format!("args[{}].parse::<i32>().unwrap()", i + 1),
            VarType::I64  => format!("args[{}].parse::<i64>().unwrap()", i + 1),
            VarType::U8   => format!("args[{}].parse::<u8>().unwrap()", i + 1),
            VarType::U16  => format!("args[{}].parse::<u16>().unwrap()", i + 1),
            VarType::U32  => format!("args[{}].parse::<u32>().unwrap()", i + 1),
            VarType::U64  => format!("args[{}].parse::<u64>().unwrap()", i + 1),
            VarType::Char => format!("args[{}].chars().next().unwrap()", i + 1),
        };
        s.push_str(&format!("    let {}: {} = {};\n",
            b.name, b.var_type.rust_type(), parse_expr));
    }
    let call_args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    s.push_str(&format!(
        "    let r = {}({}) as i64;\n",
        function_name,
        call_args.join(", ")
    ));
    s.push_str("    println!(\"{}\", r);\n}\n");
    let path = format!("/tmp/equivalence_checker/{}_rust_runner.rs", function_name);
    fs::write(&path, s)?;
    Ok(path)
}

// ── Compilation ───────────────────────────────────────

fn compile_c_runner(src: &str, out: &str) -> Result<()> {
    let o = Command::new("clang-15")
        .args(["-O0", src, "-o", out])
        .output()?;
    if !o.status.success() {
        return Err(CheckerError::CompilationError(format!(
            "C runner build failed:\n{}",
            String::from_utf8_lossy(&o.stderr)
        )).into());
    }
    Ok(())
}

fn compile_rust_runner(src: &str, out: &str) -> Result<()> {
    let o = Command::new("rustup")
        .args(["run", "1.69.0", "rustc", "-C", "opt-level=0", src, "-o", out])
        .output()?;
    if !o.status.success() {
        return Err(CheckerError::CompilationError(format!(
            "Rust runner build failed:\n{}",
            String::from_utf8_lossy(&o.stderr)
        )).into());
    }
    Ok(())
}

fn compile_c_to_ir(c_file: &str, output: &str) -> Result<()> {
    let o = Command::new("clang-15")
        .args([
            "-emit-llvm", "-c", "-O0",
            "-Xclang", "-disable-O0-optnone",
            "-fno-stack-protector",
            "-fno-inline",
            "-I/home/fathima/klee/include",
            c_file, "-o", output,
        ])
        .output()?;
    if !o.status.success() {
        return Err(CheckerError::CompilationError(format!(
            "C compilation failed:\n{}",
            String::from_utf8_lossy(&o.stderr)
        )).into());
    }
    Ok(())
}

fn compile_rust_to_ir(rust_file: &str, output: &str) -> Result<()> {
    let o = Command::new("rustup")
        .args([
            "run", "1.69.0", "rustc",
            "--emit=llvm-bc",
            "-C", "opt-level=0",
            "-C", "inline-threshold=0",
            "-C", "debuginfo=0",
            "-C", "overflow-checks=off",
            "--crate-type=lib",
            "-o", output,
            rust_file,
        ])
        .output()?;
    if !o.status.success() {
        return Err(CheckerError::CompilationError(format!(
            "Rust compilation failed:\n{}",
            String::from_utf8_lossy(&o.stderr)
        )).into());
    }
    Ok(())
}

fn verify_ir_basic(ir_path: &str) -> Result<()> {
    let meta = fs::metadata(ir_path)?;
    if meta.len() == 0 {
        return Err(CheckerError::CompilationError(format!(
            "IR file {} is empty", ir_path
        )).into());
    }
    let mut f = fs::File::open(ir_path)?;
    let mut magic = [0u8; 2];
    use std::io::Read;
    f.read_exact(&mut magic)?;
    if &magic != b"BC" {
        return Err(CheckerError::CompilationError(format!(
            "IR file {} is not valid LLVM bitcode (missing BC magic)", ir_path
        )).into());
    }
    Ok(())
}

fn emit_c_ll(c_file: &str, out_ll: &str) -> Result<()> {
    let o = Command::new("clang-15")
        .args([
            "-S", "-emit-llvm", "-O0",
            "-Xclang", "-disable-O0-optnone",
            "-fno-stack-protector", "-fno-inline",
            "-I/home/fathima/klee/include",
            c_file, "-o", out_ll,
        ])
        .output()?;
    if !o.status.success() {
        return Err(CheckerError::CompilationError(format!(
            "C .ll generation failed:\n{}",
            String::from_utf8_lossy(&o.stderr)
        )).into());
    }
    Ok(())
}

fn emit_rust_ll(rust_file: &str, out_ll: &str) -> Result<()> {
    let o = Command::new("rustup")
        .args([
            "run", "1.69.0", "rustc",
            "--emit=llvm-ir",
            "-C", "opt-level=0",
            "-C", "inline-threshold=0",
            "-C", "debuginfo=0",
            "-C", "overflow-checks=off",
            "--crate-type=lib",
            rust_file, "-o", out_ll,
        ])
        .output()?;
    if !o.status.success() {
        return Err(CheckerError::CompilationError(format!(
            "Rust .ll generation failed:\n{}",
            String::from_utf8_lossy(&o.stderr)
        )).into());
    }
    Ok(())
}

// ── Main entry point ──────────────────────────────────

pub fn compile(config: &AnalysisConfig) -> Result<IrFiles> {
    fs::create_dir_all("/tmp/equivalence_checker")?;

    // Parse typed bounds (defaults to i32 for all)
    let typed_bounds = parse_typed_bounds(config);

    // ── Harnesses ─────────────────────────────────────
    println!("  Generating KLEE harnesses...");
    let c_harness    = generate_c_harness(&config.c_file, &config.function_name, &typed_bounds)?;
    let rust_harness = generate_rust_harness(&config.rust_file, &config.function_name, &typed_bounds)?;

    // ── Runners ───────────────────────────────────────
    println!("  Generating runner programs...");
    let c_runner_src    = generate_c_runner(&config.c_file, &config.function_name, &typed_bounds)?;
    let rust_runner_src = generate_rust_runner(&config.rust_file, &config.function_name, &typed_bounds)?;

    let c_runner_bin = format!("/tmp/equivalence_checker/{}_c_runner", config.function_name);
    let rust_runner_bin = format!("/tmp/equivalence_checker/{}_rust_runner", config.function_name);

    println!("  Compiling runners...");
    compile_c_runner(&c_runner_src, &c_runner_bin)?;
    compile_rust_runner(&rust_runner_src, &rust_runner_bin)?;
    println!("    → C runner:    {}", c_runner_bin);
    println!("    → Rust runner: {}", rust_runner_bin);

    // ── Human-readable IR (.ll) ───────────────────────
    let ll_dir = "output/ir";
    fs::create_dir_all(ll_dir)?;
    let c_ll = format!("{}/{}_c_harness.ll",    ll_dir, config.function_name);
    let r_ll = format!("{}/{}_rust_harness.ll", ll_dir, config.function_name);

    println!("  Dumping human-readable LLVM IR (.ll)...");
    emit_c_ll(&c_harness, &c_ll).ok();    // non-fatal — only for debugging
    emit_rust_ll(&rust_harness, &r_ll).ok();
    println!("    → C .ll:    {}", c_ll);
    println!("    → Rust .ll: {}", r_ll);

    // ── Bitcode for KLEE ─────────────────────────────
    let c_ir_path = format!("/tmp/equivalence_checker/{}_c.bc", config.function_name);
    let rust_ir_path = format!("/tmp/equivalence_checker/{}_rust.bc", config.function_name);

    println!("  Compiling C harness to LLVM IR...");
    compile_c_to_ir(&c_harness, &c_ir_path)?;
    if !Path::new(&c_ir_path).exists() {
        return Err(CheckerError::CompilationError("C compilation produced no output".into()).into());
    }
    println!("    → Generated: {}", c_ir_path);

    println!("  Compiling Rust harness to LLVM IR...");
    compile_rust_to_ir(&rust_harness, &rust_ir_path)?;
    if !Path::new(&rust_ir_path).exists() {
        return Err(CheckerError::CompilationError("Rust compilation produced no output".into()).into());
    }
    println!("    → Generated: {}", rust_ir_path);

    println!("  Verifying IR files...");
    verify_ir_basic(&c_ir_path)?;
    verify_ir_basic(&rust_ir_path)?;
    println!("    → IR files created successfully");

    Ok(IrFiles {
        c_ir_path,
        rust_ir_path,
        c_runner_bin,
        rust_runner_bin,
    })
}