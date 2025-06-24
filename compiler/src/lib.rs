// compiler/src/codegen.rs
// compiler/src/lib.rs
//! T-Lang compiler library.
//!
//! Provides a complete compilation pipeline from source code to various target backends.
//! Designed for safety-critical systems with comprehensive error handling and analysis.

use shared::{Program, Result, TlError};
use errors::TlError as CompilerError;
use miette::SourceSpan;

pub mod parser;
pub mod types;
pub mod safety;
pub mod codegen;
pub mod backends;

// Re-export key types for convenience
pub use parser::{Parser, parse_source, parse_expression};
pub use types::{check_program, check_expression, TypeChecker};
pub use safety::{analyze_safety, SafetyAnalyzer, SafetyViolation, SafetySeverity};
pub use codegen::{CodeGenerator, GeneratedCode};

/// Main compiler pipeline that processes T-Lang source code.
pub struct Compiler {
    /// Source code being compiled
    source: String,
    /// Compilation options
    options: CompilerOptions,
    /// Collected warnings and errors
    diagnostics: Vec<CompilerDiagnostic>,
}

/// Compiler configuration options.
#[derive(Debug, Clone)]
pub struct CompilerOptions {
    /// Target backend for code generation
    pub target: String,
    /// Optimization level (0 = none, 3 = maximum)
    pub optimization_level: u8,
    /// Enable safety analysis
    pub safety_analysis: bool,
    /// Enable strict mode (treat warnings as errors)
    pub strict_mode: bool,
    /// Maximum number of errors before stopping
    pub max_errors: usize,
    /// Output directory for generated files
    pub output_dir: String,
    /// Debug information level
    pub debug_level: u8,
}

/// Compilation result containing generated code and diagnostics.
#[derive(Debug)]
pub struct CompilationResult {
    /// Generated code for the target backend
    pub code: Option<GeneratedCode>,
    /// All diagnostics (errors, warnings, info)
    pub diagnostics: Vec<CompilerDiagnostic>,
    /// Whether compilation succeeded
    pub success: bool,
}

/// Compiler diagnostic (error, warning, or info message).
#[derive(Debug, Clone)]
pub struct CompilerDiagnostic {
    /// Severity level
    pub level: DiagnosticLevel,
    /// Human-readable message
    pub message: String,
    /// Source location where the diagnostic occurred
    pub span: Option<SourceSpan>,
    /// Diagnostic code for categorization
    pub code: Option<String>,
    /// Suggested fix if available
    pub suggestion: Option<String>,
}

/// Diagnostic severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticLevel {
    Info,
    Warning,
    Error,
    Fatal,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            target: "rust".to_string(),
            optimization_level: 1,
            safety_analysis: true,
            strict_mode: false,
            max_errors: 100,
            output_dir: "target".to_string(),
            debug_level: 1,
        }
    }
}

impl Compiler {
    /// Create a new compiler instance.
    pub fn new(source: String, options: CompilerOptions) -> Self {
        Self {
            source,
            options,
            diagnostics: Vec::new(),
        }
    }

    /// Create a compiler with default options.
    pub fn with_defaults(source: String) -> Self {
        Self::new(source, CompilerOptions::default())
    }

    /// Compile the source code through the complete pipeline.
    pub fn compile(&mut self) -> CompilationResult {
        // Clear previous diagnostics
        self.diagnostics.clear();

        // Phase 1: Parsing
        let mut program = match self.parse_phase() {
            Ok(program) => program,
            Err(error) => {
                self.add_error_diagnostic(error);
                return self.create_failed_result();
            }
        };

        // Phase 2: Type checking
        if let Err(error) = self.type_check_phase(&mut program) {
            self.add_error_diagnostic(error);
            if self.options.strict_mode {
                return self.create_failed_result();
            }
        }

        // Phase 3: Safety analysis
        if self.options.safety_analysis {
            if let Err(error) = self.safety_analysis_phase(&program) {
                self.add_error_diagnostic(error);
                if self.options.strict_mode {
                    return self.create_failed_result();
                }
            }
        }

        // Phase 4: Code generation
        let generated_code = match self.codegen_phase(&program) {
            Ok(code) => Some(code),
            Err(error) => {
                self.add_error_diagnostic(error);
                return self.create_failed_result();
            }
        };

        // Return successful result
        CompilationResult {
            code: generated_code,
            diagnostics: self.diagnostics.clone(),
            success: !self.has_errors(),
        }
    }

    /// Parse the source code into an AST.
    fn parse_phase(&mut self) -> Result<Program> {
        let parser = Parser::new(self.source.clone());
        parser.parse()
    }

    /// Perform type checking and inference.
    fn type_check_phase(&mut self, program: &mut Program) -> Result<()> {
        let mut type_checker = TypeChecker::new(self.source.clone());
        type_checker.check_program(program)
    }

    /// Perform safety analysis.
    fn safety_analysis_phase(&mut self, program: &Program) -> Result<()> {
        let violations = analyze_safety(program, self.source.clone())?;

        // Convert safety violations to diagnostics
        for violation in violations {
            let diagnostic = CompilerDiagnostic {
                level: match violation.severity() {
                    SafetySeverity::Info => DiagnosticLevel::Info,
                    SafetySeverity::Warning => DiagnosticLevel::Warning,
                    SafetySeverity::Error => DiagnosticLevel::Error,
                    SafetySeverity::Critical => DiagnosticLevel::Fatal,
                },
                message: violation.description(),
                span: Some(self.get_violation_span(&violation)),
                code: Some(self.get_violation_code(&violation)),
                suggestion: None,
            };

            self.diagnostics.push(diagnostic);
        }

        Ok(())
    }

    /// Generate code for the target backend.
    fn codegen_phase(&mut self, program: &Program) -> Result<GeneratedCode> {
        let mut generator = CodeGenerator::new(
            self.options.target.clone(),
            self.options.optimization_level,
        );

        generator.generate(program)
    }

    // Helper methods

    fn add_error_diagnostic(&mut self, error: TlError) {
        let diagnostic = CompilerDiagnostic {
            level: DiagnosticLevel::Error,
            message: error.to_string(),
            span: self.extract_span_from_error(&error),
            code: Some(self.extract_code_from_error(&error)),
            suggestion: None,
        };

        self.diagnostics.push(diagnostic);
    }

    fn create_failed_result(&self) -> CompilationResult {
        CompilationResult {
            code: None,
            diagnostics: self.diagnostics.clone(),
            success: false,
        }
    }

    fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| {
            matches!(d.level, DiagnosticLevel::Error | DiagnosticLevel::Fatal)
        })
    }

    fn extract_span_from_error(&self, error: &TlError) -> Option<SourceSpan> {
        match error {
            TlError::Lexer { span, .. } => Some(*span),
            TlError::Parser { span, .. } => Some(*span),
            TlError::Type { span, .. } => Some(*span),
            TlError::Safety { span, .. } => Some(*span),
            TlError::Runtime { span, .. } => Some(*span),
            _ => None,
        }
    }

    fn extract_code_from_error(&self, error: &TlError) -> String {
        match error {
            TlError::Lexer { .. } => "E0001".to_string(),
            TlError::Parser { .. } => "E0002".to_string(),
            TlError::Type { .. } => "E0003".to_string(),
            TlError::Safety { .. } => "E0004".to_string(),
            TlError::Runtime { .. } => "E0005".to_string(),
            TlError::Io { .. } => "E0006".to_string(),
            TlError::Internal { .. } => "E0999".to_string(),
        }
    }

    fn get_violation_span(&self, violation: &SafetyViolation) -> SourceSpan {
        match violation {
            SafetyViolation::UninitializedVariable { span, .. } => *span,
            SafetyViolation::UseAfterMove { span, .. } => *span,
            SafetyViolation::MemoryLeak { allocation_site, .. } => *allocation_site,
            SafetyViolation::ResourceLeak { acquisition_site, .. } => *acquisition_site,
            SafetyViolation::NullPointerDereference { span, .. } => *span,
            SafetyViolation::BufferOverflow { span, .. } => *span,
            SafetyViolation::StackOverflow { span, .. } => *span,
            SafetyViolation::UnsafeOperation { span, .. } => *span,
            SafetyViolation::DataRace { span, .. } => *span,
            SafetyViolation::RealtimeViolation { span, .. } => *span,
        }
    }

    fn get_violation_code(&self, violation: &SafetyViolation) -> String {
        match violation {
            SafetyViolation::UninitializedVariable { .. } => "S0001".to_string(),
            SafetyViolation::UseAfterMove { .. } => "S0002".to_string(),
            SafetyViolation::MemoryLeak { .. } => "S0003".to_string(),
            SafetyViolation::ResourceLeak { .. } => "S0004".to_string(),
            SafetyViolation::NullPointerDereference { .. } => "S0005".to_string(),
            SafetyViolation::BufferOverflow { .. } => "S0006".to_string(),
            SafetyViolation::StackOverflow { .. } => "S0007".to_string(),
            SafetyViolation::UnsafeOperation { .. } => "S0008".to_string(),
            SafetyViolation::DataRace { .. } => "S0009".to_string(),
            SafetyViolation::RealtimeViolation { .. } => "S0010".to_string(),
        }
    }
}

impl CompilerDiagnostic {
    /// Create a new error diagnostic.
    pub fn error(message: String, span: Option<SourceSpan>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message,
            span,
            code: None,
            suggestion: None,
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(message: String, span: Option<SourceSpan>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message,
            span,
            code: None,
            suggestion: None,
        }
    }

    /// Create a new info diagnostic.
    pub fn info(message: String, span: Option<SourceSpan>) -> Self {
        Self {
            level: DiagnosticLevel::Info,
            message,
            span,
            code: None,
            suggestion: None,
        }
    }

    /// Add a suggestion to this diagnostic.
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Add a diagnostic code.
    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }
}

/// Convenience function to compile source code with default options.
pub fn compile_source(source: String) -> CompilationResult {
    let mut compiler = Compiler::with_defaults(source);
    compiler.compile()
}

/// Convenience function to compile source code with specific target.
pub fn compile_to_target(source: String, target: String) -> CompilationResult {
    let mut options = CompilerOptions::default();
    options.target = target;

    let mut compiler = Compiler::new(source, options);
    compiler.compile()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple_program() {
        let source = r#"
            fn main() {
                let x = 42;
                print("Hello, T-Lang!");
            }
        "#.to_string();

        let result = compile_source(source);

        // Should compile successfully (may have warnings)
        if !result.success {
            for diagnostic in &result.diagnostics {
                println!("{:?}: {}", diagnostic.level, diagnostic.message);
            }
        }

        // At minimum, should not have fatal errors
        assert!(!result.diagnostics.iter().any(|d| d.level == DiagnosticLevel::Fatal));
    }

    #[test]
    fn test_compile_with_type_error() {
        let source = r#"
            fn main() {
                let x: i32 = "hello";
            }
        "#.to_string();

        let result = compile_source(source);

        // Should have type error
        assert!(result.diagnostics.iter().any(|d| {
            d.level == DiagnosticLevel::Error && d.message.contains("type")
        }));
    }

    #[test]
    fn test_compile_with_safety_violation() {
        let source = r#"
            fn main() {
                let x: i32;
                let y = x; // Use of uninitialized variable
            }
        "#.to_string();

        let mut options = CompilerOptions::default();
        options.safety_analysis = true;

        let mut compiler = Compiler::new(source, options);
        let result = compiler.compile();

        // Should have safety violation
        assert!(result.diagnostics.iter().any(|d| {
            d.message.contains("uninitialized")
        }));
    }
}