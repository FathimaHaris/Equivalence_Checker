use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub c_file: String,
    pub rust_file: String,
    pub function_name: String,
    pub bounds: Vec<InputBound>,
    pub max_paths: u32,
    pub timeout: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
    Integer,
    Float,
}

impl Default for ParamType {
    fn default() -> Self { ParamType::Integer }
}

#[derive(Debug, Clone)]
pub struct InputBound {
    pub name: String,
    pub min: i64,
    pub max: i64,
    pub param_type: ParamType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReturnKind {
    Integer,
    Float,
    Bool,
    Void,
}

impl Default for ReturnKind {
    fn default() -> Self { ReturnKind::Integer }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquivalenceDetail {
    pub inputs_tested: u32,
    pub return_value_match: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivergingInput {
    pub name: String,
    pub value: i64,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSummary {
    pub id: String,
    pub program: ProgramKind,
    pub constraints: Vec<String>,
    pub return_expr: Option<String>,
    pub witness: Vec<(String, i64)>,
    pub observables: ObservableEffects,
    #[serde(default)]
    pub label_map: HashMap<String, String>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceResult {
    pub verdict: Verdict,
    pub paths_compared: u32,
    pub counterexample: Option<Counterexample>,
    pub time_taken: f64,
    pub statistics: CheckerStatistics,
    pub c_path: Option<PathSummary>,
    pub rust_path: Option<PathSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Verdict {
    Equivalent,
    NotEquivalent,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterexample {
    pub inputs: Vec<(String, i64)>,
    pub input_strings: Vec<(String, String)>,
    pub c_behavior: ConcreteBehavior,
    pub rust_behavior: ConcreteBehavior,
    pub differences: Vec<Difference>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CheckerStatistics {
    pub total_paths_c: usize,
    pub total_paths_rust: usize,
    pub merged_pairs: usize,
    pub z3_queries: u32,
    pub z3_time_ms: u64,
}

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