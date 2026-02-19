// src/compiler/mod.rs — SKELETON
use crate::types::AnalysisConfig;
use anyhow::Result;

pub struct IrFiles {
    pub c_ir_path:    String,
    pub rust_ir_path: String,
}

pub fn compile(config: &AnalysisConfig) -> Result<IrFiles> {
    Ok(IrFiles {
        c_ir_path:    format!("/tmp/{}.bc", config.function_name),
        rust_ir_path: format!("/tmp/{}_rust.bc", config.function_name),
    })
}