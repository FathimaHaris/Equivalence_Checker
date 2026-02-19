// src/reporter/mod.rs — SKELETON
use crate::types::{AnalysisConfig, EquivalenceResult};
use anyhow::Result;

pub fn generate(_config: &AnalysisConfig, _result: &EquivalenceResult) -> Result<String> {
    let path = "output/report.html".to_string();
    Ok(path)
}