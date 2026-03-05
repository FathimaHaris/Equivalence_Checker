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
    pub name: String,
    pub min: i64,
    pub max: i64,
}

// ───────────────────────────────────────────────────────
// VALIDATION TYPES
// ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub success: bool,
    pub c_signature: Option<FunctionSignature>,
    pub rust_signature: Option<FunctionSignature>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<String>,
    pub return_type: String,
}

// ───────────────────────────────────────────────────────
// PATH SUMMARY TYPES (from KLEE)
// ───────────────────────────────────────────────────────

/// Complete symbolic path summary for ONE execution path
/// This matches your DFD2_symbolic.jpg - output of step 0.5.4
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSummary {
    /// Unique ID for this path
    pub id: String,
    /// Which program: C or Rust
    pub program: ProgramKind,
    /// Path constraints in SMT-LIB format (from KLEE .kquery)
    pub constraints: Vec<String>,
    /// Return expression in SMT-LIB format
    pub return_expr: Option<String>,
    /// Concrete witness values from KLEE
    pub witness: Vec<(String, i64)>,
    /// Observable effects (stdout, stderr, globals)
    pub observables: ObservableEffects,
}

/// Observable effects captured during instrumentation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObservableEffects {
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub global_writes: Vec<(String, String)>,
    pub file_ops: Vec<FileOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub op_type: String,
    pub filename: String,
    pub data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProgramKind {
    C,
    Rust,
}

// ───────────────────────────────────────────────────────
// EQUIVALENCE CHECKER TYPES (for Z3)
// ───────────────────────────────────────────────────────

/// Result of equivalence checking (matches your DFD)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceResult {
    pub verdict: Verdict,
    pub paths_compared: u32,
    pub counterexample: Option<Counterexample>,
    pub time_taken: f64,
    pub statistics: CheckerStatistics,
    pub c_path:  Option<PathSummary>,   
    pub rust_path: Option<PathSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Verdict {
    Equivalent,      // UNSAT - programs are equivalent
    NotEquivalent,   // SAT - found counterexample
    Unknown,         // Z3 could not decide
}

/// Counterexample found by Z3 (SAT result)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterexample {
    pub inputs: Vec<(String, i64)>,
    pub c_behavior: ConcreteBehavior,
    pub rust_behavior: ConcreteBehavior,
    pub differences: Vec<Difference>,
}

/// Concrete program behavior for specific inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcreteBehavior {
    pub return_value: String,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
    pub globals: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    pub kind: DifferenceKind,
    pub c_value: String,
    pub rust_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifferenceKind {
    ReturnValue,
    Stdout,
    Stderr,
    GlobalVariable(String),
    FileOperation,
}

/// Statistics from the checker
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CheckerStatistics {
    pub total_paths_c: usize,
    pub total_paths_rust: usize,
    pub merged_pairs: usize,
    pub z3_queries: u32,
    pub z3_time_ms: u64,
}

// ───────────────────────────────────────────────────────
// ERROR TYPES
// ───────────────────────────────────────────────────────

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
    #[error("Z3 solver error: {0}")]
    Z3Error(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}