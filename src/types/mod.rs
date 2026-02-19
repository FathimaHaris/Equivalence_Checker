// src/types/mod.rs
// ═══════════════════════════════════════════════════════
// Shared data types used by ALL modules
// ═══════════════════════════════════════════════════════

use serde::{Deserialize, Serialize};

// ───────────────────────────────────────────────────────
// INPUT TYPES
// ───────────────────────────────────────────────────────

/// Everything the user provides as input
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    /// Path to C source file
    pub c_file: String,

    /// Path to Rust source file  
    pub rust_file: String,

    /// Name of function to check
    pub function_name: String,

    /// Input bounds for symbolic execution
    pub bounds: Vec<InputBound>,

    /// Maximum paths KLEE should explore
    pub max_paths: u32,

    /// Maximum time in seconds
    pub timeout: u32,
}

/// Bound for one input variable
#[derive(Debug, Clone)]
pub struct InputBound {
    /// Variable name e.g. "x"
    pub name: String,

    /// Minimum value e.g. 0
    pub min: i64,

    /// Maximum value e.g. 100
    pub max: i64,
}

// ───────────────────────────────────────────────────────
// VALIDATION TYPES
// ───────────────────────────────────────────────────────

/// Result of validating input files
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Did validation pass?
    pub success: bool,

    /// C function signature found
    pub c_signature: Option<FunctionSignature>,

    /// Rust function signature found
    pub rust_signature: Option<FunctionSignature>,

    /// Any errors found
    pub errors: Vec<String>,
}

/// A function's signature
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: String,
}

// ───────────────────────────────────────────────────────
// PATH SUMMARY TYPES (output of KLEE symbolic execution)
// ───────────────────────────────────────────────────────

/// Complete symbolic path summary for ONE execution path
/// This is what KLEE produces for each path it explores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSummary {
    /// Unique ID for this path e.g. "C-1", "R-2"
    pub id: String,

    /// Which program: C or Rust
    pub program: ProgramKind,

    /// The path condition (constraints that led here)
    /// e.g. ["x > 10", "y >= 0"]
    pub path_condition: Vec<String>,

    /// The return expression (symbolic)
    /// e.g. "x + y"
    pub return_expr: String,

    /// Stdout strings logged by instrumentation hooks
    /// e.g. ["Hello\n", "Done\n"]
    pub stdout_log: Vec<String>,

    /// Stderr strings logged
    pub stderr_log: Vec<String>,

    /// Global variable writes logged
    /// e.g. [("counter", "counter_0 + 1")]
    pub global_writes: Vec<(String, String)>,

    /// File operations logged
    pub file_ops: Vec<FileOperation>,
}

/// Which program a path belongs to
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgramKind {
    C,
    Rust,
}

/// A file operation that was logged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub op_type: String,   // "open", "write", "close"
    pub filename: String,
    pub data: Option<String>,
}

// ───────────────────────────────────────────────────────
// EQUIVALENCE RESULT TYPES (output of Z3)
// ───────────────────────────────────────────────────────

/// Final verdict from Z3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceResult {
    /// Overall verdict
    pub verdict: Verdict,

    /// Paths that were compared
    pub paths_compared: u32,

    /// If not equivalent: the counterexample
    pub counterexample: Option<Counterexample>,

    /// Time taken in seconds
    pub time_taken: f64,
}

/// The verdict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Verdict {
    /// Z3 proved UNSAT — programs are equivalent
    Equivalent,

    /// Z3 found SAT — programs differ
    NotEquivalent,

    /// Z3 could not decide within timeout
    Unknown,
}

/// A concrete counterexample found by Z3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterexample {
    /// The concrete input values
    /// e.g. [("x", 10), ("y", 5)]
    pub inputs: Vec<(String, i64)>,

    /// What C program does with these inputs
    pub c_behavior: BehaviorSnapshot,

    /// What Rust program does with these inputs
    pub rust_behavior: BehaviorSnapshot,

    /// What specifically differs
    pub differences: Vec<Difference>,
}

/// What a program does for specific inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSnapshot {
    pub return_value: String,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub globals: Vec<(String, String)>,
}

/// One specific difference found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    /// What kind of difference
    pub kind: DifferenceKind,

    /// C value
    pub c_value: String,

    /// Rust value
    pub rust_value: String,
}

/// Kind of difference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifferenceKind {
    ReturnValue,
    Stdout,
    Stderr,
    GlobalVariable(String),
    FileOperation,
}

// ───────────────────────────────────────────────────────
// ERROR TYPES
// ───────────────────────────────────────────────────────

/// All possible errors in our tool
#[derive(Debug, thiserror::Error)]
pub enum CheckerError {
    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Compilation failed: {0}")]
    CompilationError(String),

    #[error("Normalization failed: {0}")]
    NormalizationError(String),

    #[error("Instrumentation failed: {0}")]
    InstrumentationError(String),

    #[error("Symbolic execution failed: {0}")]
    SymbolicExecutionError(String),

    #[error("Equivalence checking failed: {0}")]
    EquivalenceError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}