// src/compiler/mod.rs
// ═══════════════════════════════════════════════════════
// Module 2: LLVM IR Compilation
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

// ── Harness generation ────────────────────────────────

fn generate_c_harness(
    c_file: &str,
    function_name: &str,
    bounds: &[crate::types::InputBound],
) -> Result<String> {
    println!("    Generating C harness with KLEE directives...");
    let content = fs::read_to_string(c_file)?;

    let mut h = String::new();
    h.push_str("#include <klee/klee.h>\n\n");
    h.push_str("// Original function\n");
    h.push_str(&content);
    h.push_str("\n\n// Auto-generated KLEE harness\n");
    h.push_str("int main() {\n");

    for b in bounds {
        h.push_str(&format!("    int {};\n", b.name));
    }
    h.push('\n');

    for b in bounds {
        h.push_str(&format!(
            "    klee_make_symbolic(&{name}, sizeof({name}), \"{name}\");\n",
            name = b.name
        ));
    }
    h.push('\n');

    for b in bounds {
        h.push_str(&format!(
            "    klee_assume({name} >= {min} && {name} <= {max});\n",
            name = b.name,
            min  = b.min,
            max  = b.max
        ));
    }
    h.push('\n');

    let args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    h.push_str(&format!(
        "    volatile int result = {fn_name}({args});\n",
        fn_name = function_name,
        args    = args.join(", ")
    ));
    h.push_str("    return result;\n");
    h.push_str("}\n");

    let path = format!("/tmp/equivalence_checker/{}_c_harness.c", function_name);
    fs::write(&path, h)?;
    Ok(path)
}

fn generate_rust_harness(
    rust_file: &str,
    function_name: &str,
    bounds: &[crate::types::InputBound],
) -> Result<String> {
    println!("    Generating Rust harness with KLEE FFI...");
    let content = fs::read_to_string(rust_file)?;

    let mut h = String::new();
    h.push_str("use std::os::raw::c_void;\n\n");
    h.push_str("extern \"C\" {\n");
    h.push_str("    fn klee_make_symbolic(addr: *mut c_void, nbytes: usize, name: *const u8);\n");
    h.push_str("    fn klee_assume(cond: i32);\n");
    h.push_str("}\n\n");
    h.push_str(&content);
    h.push_str("\n\n");
    h.push_str("#[no_mangle]\n");
    h.push_str("pub extern \"C\" fn klee_harness() -> i32 {\n");

    for b in bounds {
        h.push_str(&format!("    let mut {}: i32 = 0;\n", b.name));
    }
    h.push('\n');

    h.push_str("    unsafe {\n");
    for b in bounds {
        h.push_str(&format!(
            "        klee_make_symbolic(\n            &mut {name} as *mut i32 as *mut c_void,\n            std::mem::size_of::<i32>(),\n            b\"{name}\\0\".as_ptr()\n        );\n",
            name = b.name
        ));
    }
    for b in bounds {
        h.push_str(&format!(
            "        klee_assume(({name} >= {min} && {name} <= {max}) as i32);\n",
            name = b.name,
            min  = b.min,
            max  = b.max
        ));
    }
    h.push_str("    }\n\n");

    let args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    h.push_str(&format!(
        "    {fn_name}({args})\n",
        fn_name = function_name,
        args    = args.join(", ")
    ));
    h.push_str("}\n");

    let path = format!(
        "/tmp/equivalence_checker/{}_rust_harness.rs",
        function_name
    );
    fs::write(&path, h)?;
    Ok(path)
}

// ── Runner generation (for concrete differential testing) ──

fn generate_c_runner(
    c_file: &str,
    function_name: &str,
    bounds: &[crate::types::InputBound],
) -> Result<String> {
    let content = fs::read_to_string(c_file)?;
    let mut s = String::new();
    s.push_str("#include <stdio.h>\n#include <stdlib.h>\n\n");
    s.push_str(&content);
    s.push_str("\n\nint main(int argc, char** argv) {\n");
    s.push_str(&format!("    if (argc != {}) return 2;\n", bounds.len() + 1));
    for (i, b) in bounds.iter().enumerate() {
        s.push_str(&format!("    int {} = atoi(argv[{}]);\n", b.name, i + 1));
    }
    let args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    s.push_str(&format!(
        "    int r = {}({});\n",
        function_name,
        args.join(", ")
    ));
    s.push_str("    printf(\"%d\\n\", r);\n    return 0;\n}\n");
    let path = format!("/tmp/equivalence_checker/{}_c_runner.c", function_name);
    fs::write(&path, s)?;
    Ok(path)
}

fn generate_rust_runner(
    rust_file: &str,
    function_name: &str,
    bounds: &[crate::types::InputBound],
) -> Result<String> {
    let content = fs::read_to_string(rust_file)?;
    let mut s = String::new();
    s.push_str("use std::env;\n\n");
    s.push_str(&content);
    s.push_str("\n\nfn main() {\n");
    s.push_str("    let args: Vec<String> = env::args().collect();\n");
    s.push_str(&format!(
        "    if args.len() != {} {{ std::process::exit(2); }}\n",
        bounds.len() + 1
    ));
    for (i, b) in bounds.iter().enumerate() {
        s.push_str(&format!(
            "    let {}: i32 = args[{}].parse().unwrap();\n",
            b.name,
            i + 1
        ));
    }
    let call_args: Vec<String> = bounds.iter().map(|b| b.name.clone()).collect();
    s.push_str(&format!(
        "    let r = {}({});\n",
        function_name,
        call_args.join(", ")
    ));
    s.push_str("    println!(\"{}\", r);\n}\n");
    let path = format!(
        "/tmp/equivalence_checker/{}_rust_runner.rs",
        function_name
    );
    fs::write(&path, s)?;
    Ok(path)
}

fn compile_c_runner(src: &str, out: &str) -> Result<()> {
    let o = Command::new("clang-15")
        .args(["-O0", src, "-o", out])
        .output()?;
    if !o.status.success() {
        return Err(CheckerError::CompilationError(format!(
            "C runner build failed:\n{}",
            String::from_utf8_lossy(&o.stderr)
        ))
        .into());
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
        ))
        .into());
    }
    Ok(())
}

// ── LLVM IR compilation ───────────────────────────────

/// Compile C harness → LLVM bitcode for KLEE.
///
/// Key flags:
///   -O0 -Xclang -disable-O0-optnone   keep branches symbolic
///   -fno-inline                         don't inline the target fn
///   NO opt passes after this step       normalization must not touch KLEE bc
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
        ))
        .into());
    }
    Ok(())
}

/// Compile Rust harness → LLVM bitcode for KLEE.
///
/// IMPORTANT: we use --emit=llvm-bc with overflow-checks=off so that
/// branch structure matches C.  The crate-type=lib is required to avoid
/// Rust injecting its own main() which confuses KLEE.
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
        ))
        .into());
    }
    Ok(())
}

fn verify_ir_basic(ir_path: &str) -> Result<()> {
    let meta = fs::metadata(ir_path)?;
    if meta.len() == 0 {
        return Err(CheckerError::CompilationError(format!(
            "IR file {} is empty",
            ir_path
        ))
        .into());
    }
    let mut f = fs::File::open(ir_path)?;
    let mut magic = [0u8; 2];
    use std::io::Read;
    f.read_exact(&mut magic)?;
    if &magic != b"BC" {
        return Err(CheckerError::CompilationError(format!(
            "IR file {} is not valid LLVM bitcode (missing BC magic)",
            ir_path
        ))
        .into());
    }
    Ok(())
}

fn emit_c_ll(c_file: &str, out_ll: &str) -> Result<()> {
    let o = Command::new("clang-15")
        .args([
            "-S", "-emit-llvm", "-O0",
            "-Xclang", "-disable-O0-optnone",
            "-fno-stack-protector",
            "-fno-inline",
            "-I/home/fathima/klee/include",
            c_file, "-o", out_ll,
        ])
        .output()?;
    if !o.status.success() {
        return Err(CheckerError::CompilationError(format!(
            "C .ll generation failed:\n{}",
            String::from_utf8_lossy(&o.stderr)
        ))
        .into());
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
        ))
        .into());
    }
    Ok(())
}

// ── Main entry point ──────────────────────────────────

pub fn compile(config: &AnalysisConfig) -> Result<IrFiles> {
    fs::create_dir_all("/tmp/equivalence_checker")?;

    // ── Harnesses ─────────────────────────────────────
    println!("  Generating KLEE harnesses...");
    let c_harness = generate_c_harness(
        &config.c_file,
        &config.function_name,
        &config.bounds,
    )?;
    let rust_harness = generate_rust_harness(
        &config.rust_file,
        &config.function_name,
        &config.bounds,
    )?;

    // ── Runners ───────────────────────────────────────
    println!("  Generating runner programs...");
    let c_runner_src =
        generate_c_runner(&config.c_file, &config.function_name, &config.bounds)?;
    let rust_runner_src =
        generate_rust_runner(&config.rust_file, &config.function_name, &config.bounds)?;

    let c_runner_bin = format!(
        "/tmp/equivalence_checker/{}_c_runner",
        config.function_name
    );
    let rust_runner_bin = format!(
        "/tmp/equivalence_checker/{}_rust_runner",
        config.function_name
    );

    println!("  Compiling runners...");
    compile_c_runner(&c_runner_src, &c_runner_bin)?;
    compile_rust_runner(&rust_runner_src, &rust_runner_bin)?;
    println!("    → C runner:    {}", c_runner_bin);
    println!("    → Rust runner: {}", rust_runner_bin);

    // ── Human-readable IR (.ll) ───────────────────────
    let ll_dir = "output/ir";
    fs::create_dir_all(ll_dir)?;
    let c_ll   = format!("{}/{}_c_harness.ll",    ll_dir, config.function_name);
    let r_ll   = format!("{}/{}_rust_harness.ll", ll_dir, config.function_name);

    println!("  Dumping human-readable LLVM IR (.ll)...");
    emit_c_ll(&c_harness, &c_ll)?;
    emit_rust_ll(&rust_harness, &r_ll)?;
    println!("    → C .ll:    {}", c_ll);
    println!("    → Rust .ll: {}", r_ll);

    // ── Bitcode for KLEE ─────────────────────────────
    // CRITICAL: these bitcode files go directly to KLEE.
    // The normalizer must NOT run opt passes on them because
    // opt's mem2reg/dce can eliminate klee_make_symbolic calls,
    // reducing path count to 1.
    let c_ir_path = format!(
        "/tmp/equivalence_checker/{}_c.bc",
        config.function_name
    );
    let rust_ir_path = format!(
        "/tmp/equivalence_checker/{}_rust.bc",
        config.function_name
    );

    println!("  Compiling C harness to LLVM IR...");
    compile_c_to_ir(&c_harness, &c_ir_path)?;
    if !Path::new(&c_ir_path).exists() {
        return Err(CheckerError::CompilationError(
            "C compilation produced no output".into(),
        )
        .into());
    }
    println!("    → Generated: {}", c_ir_path);

    println!("  Compiling Rust harness to LLVM IR...");
    compile_rust_to_ir(&rust_harness, &rust_ir_path)?;
    if !Path::new(&rust_ir_path).exists() {
        return Err(CheckerError::CompilationError(
            "Rust compilation produced no output".into(),
        )
        .into());
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