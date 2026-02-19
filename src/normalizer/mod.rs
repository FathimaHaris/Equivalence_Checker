// src/normalizer/mod.rs — SKELETON
use crate::types::AnalysisConfig;
use crate::compiler::IrFiles;
use anyhow::Result;

pub struct NormalizedFiles {
    pub c_normalized_path:    String,
    pub rust_normalized_path: String,
}

pub fn normalize(_config: &AnalysisConfig, ir: &IrFiles) -> Result<NormalizedFiles> {
    Ok(NormalizedFiles {
        c_normalized_path:    ir.c_ir_path.clone(),
        rust_normalized_path: ir.rust_ir_path.clone(),
    })
}