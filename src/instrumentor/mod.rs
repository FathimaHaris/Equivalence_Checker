// src/instrumentor/mod.rs — SKELETON
use crate::types::AnalysisConfig;
use crate::normalizer::NormalizedFiles;
use anyhow::Result;

pub struct InstrumentedFiles {
    pub c_instrumented_path:    String,
    pub rust_instrumented_path: String,
}

pub fn instrument(_config: &AnalysisConfig, norm: &NormalizedFiles) -> Result<InstrumentedFiles> {
    Ok(InstrumentedFiles {
        c_instrumented_path:    norm.c_normalized_path.clone(),
        rust_instrumented_path: norm.rust_normalized_path.clone(),
    })
}