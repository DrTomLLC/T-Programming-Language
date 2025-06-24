use shared::{Program, Result};
use miette::SourceSpan;

#[derive(Debug, Clone)]
pub enum SafetyViolation {
    UninitializedVariable { span: SourceSpan, name: String },
    UseAfterMove { span: SourceSpan, var: String },
    // Add others as needed
}

#[derive(Debug, Clone, Copy)]
pub enum SafetySeverity { Info, Warning, Error, Critical }

impl SafetyViolation {
    pub fn severity(&self) -> SafetySeverity { SafetySeverity::Warning }
    pub fn description(&self) -> String { "Safety violation".to_string() }
}

pub struct SafetyAnalyzer;

pub fn analyze_safety(_program: &Program, _source: String) -> Result<Vec<SafetyViolation>> {
    Ok(Vec::new()) // Stub for now
}