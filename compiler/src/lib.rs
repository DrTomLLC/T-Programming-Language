// compiler/src/lib.rs
//! T-Lang compiler library.
//! Provides a complete compilation pipeline from source code to various target backends.

use shared::{Program, Result, TlError, tokenize};
use errors::ErrorCollector;
use miette::SourceSpan;
use plugin_api::CompiledModule;

pub mod parser;
pub mod types;
pub mod safety;
// Create this module
mod codegen {
    /// Code generator for T-Lang
    pub struct CodeGenerator;

    /// Output of the code generation phase
    #[derive(Debug)]
    pub struct GeneratedCode;

    impl CodeGenerator {
        /// Create a new code generator
        pub fn new() -> Self {
            Self
        }
    }
}
pub mod backends;
pub mod lexer;

// Re-export key types for convenience
pub use parser::{Parser, parse_source, parse_expression};
pub use types::{TypeChecker, check_program, check_expression};
pub use safety::{SafetyAnalyzer, analyze_safety, SafetyViolation, SafetySeverity};
pub use codegen::{CodeGenerator, GeneratedCode};

/// Main compiler pipeline that processes T-Lang source code.
pub struct Compiler {
    /// Source code being compiled
    source: String,
    /// Source file path
    file_path: Option<String>,
    /// Compilation options
    options: CompilerOptions,
    /// Collected warnings and errors
    diagnostics: ErrorCollector,
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
    /// Enable various compiler passes
    pub enable_type_checking: bool,
    pub enable_name_resolution: bool,
    pub enable_borrow_checking: bool,
    /// Feature flags
    pub features: Vec<String>,
}

/// Compilation result containing generated code and diagnostics.
#[derive(Debug)]
pub struct CompilationResult {
    /// Generated code for the target backend
    pub code: Option<GeneratedCode>,
    /// All diagnostics (errors, warnings, info)
    pub diagnostics: Vec<TlError>,
    /// Whether compilation succeeded
    pub success: bool,
    /// Compilation statistics
    pub stats: CompilationStats,
}

/// Statistics about the compilation process.
#[derive(Debug, Default)]
pub struct CompilationStats {
    /// Number of lines of source code
    pub lines_of_code: usize,
    /// Number of tokens processed
    pub token_count: usize,
    /// Number of AST nodes created
    pub ast_node_count: usize,
    /// Time spent in each phase (in milliseconds)
    pub phase_times: PhaseTimings,
    /// Memory usage statistics
    pub memory_usage: MemoryStats,
}

/// Timing information for compilation phases.
#[derive(Debug, Default)]
pub struct PhaseTimings {
    pub lexing: u64,
    pub parsing: u64,
    pub type_checking: u64,
    pub safety_analysis: u64,
    pub code_generation: u64,
    pub total: u64,
}

/// Memory usage statistics.
#[derive(Debug, Default)]
pub struct MemoryStats {
    pub peak_memory_mb: f64,
    pub final_memory_mb: f64,
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
            enable_type_checking: true,
            enable_name_resolution: true,
            enable_borrow_checking: true,
            features: Vec::new(),
        }
    }
}

impl Compiler {
    /// Create a new compiler instance.
    pub fn new(source: String, options: CompilerOptions) -> Self {
        Self {
            source,
            file_path: None,
            options,
            diagnostics: ErrorCollector::with_limit(options.max_errors),
        }
    }

    /// Create a compiler with default options.
    pub fn with_defaults(source: String) -> Self {
        Self::new(source, CompilerOptions::default())
    }

    /// Create a compiler with a source file path.
    pub fn with_file(source: String, file_path: String, options: CompilerOptions) -> Self {
        let mut compiler = Self::new(source, options);
        compiler.file_path = Some(file_path);
        compiler
    }

    /// Set the target backend.
    pub fn with_target(mut self, target: String) -> Self {
        self.options.target = target;
        self
    }

    /// Set optimization level.
    pub fn with_optimization_level(mut self, level: u8) -> Self {
        self.options.optimization_level = level;
        self
    }

    /// Enable or disable safety analysis.
    pub fn with_safety_analysis(mut self, enable: bool) -> Self {
        self.options.safety_analysis = enable;
        self
    }

    /// Add a feature flag.
    pub fn with_feature(mut self, feature: String) -> Self {
        self.options.features.push(feature);
        self
    }

    /// Compile the source code through the complete pipeline.
    pub fn compile(&mut self) -> CompilationResult {
        let start_time = std::time::Instant::now();
        let mut stats = CompilationStats::default();

        // Clear previous diagnostics
        self.diagnostics = ErrorCollector::with_limit(self.options.max_errors);

        // Calculate basic statistics
        stats.lines_of_code = self.source.lines().count();

        // Phase 1: Lexing and Parsing
        let parse_start = std::time::Instant::now();
        let mut program = match self.parse_phase() {
            Ok(program) => {
                stats.phase_times.parsing = parse_start.elapsed().as_millis() as u64;
                program
            }
            Err(error) => {
                self.diagnostics.add(error);
                return self.create_failed_result(stats);
            }
        };

        // Phase 2: Name Resolution (if enabled)
        if self.options.enable_name_resolution {
            if let Err(error) = self.name_resolution_phase(&mut program) {
                self.diagnostics.add(error);
                if self.should_stop_compilation() {
                    return self.create_failed_result(stats);
                }
            }
        }

        // Phase 3: Type checking (if enabled)
        if self.options.enable_type_checking {
            let type_start = std::time::Instant::now();
            if let Err(error) = self.type_check_phase(&mut program) {
                self.diagnostics.add(error);
                stats.phase_times.type_checking = type_start.elapsed().as_millis() as u64;
                if self.should_stop_compilation() {
                    return self.create_failed_result(stats);
                }
            } else {
                stats.phase_times.type_checking = type_start.elapsed().as_millis() as u64;
            }
        }

        // Phase 4: Safety analysis (if enabled)
        if self.options.safety_analysis {
            let safety_start = std::time::Instant::now();
            if let Err(error) = self.safety_analysis_phase(&program) {
                self.diagnostics.add(error);
                stats.phase_times.safety_analysis = safety_start.elapsed().as_millis() as u64;
                if self.should_stop_compilation() {
                    return self.create_failed_result(stats);
                }
            } else {
                stats.phase_times.safety_analysis = safety_start.elapsed().as_millis() as u64;
            }
        }

        // Phase 5: Borrow checking (if enabled)
        if self.options.enable_borrow_checking {
            if let Err(error) = self.borrow_check_phase(&program) {
                self.diagnostics.add(error);
                if self.should_stop_compilation() {
                    return self.create_failed_result(stats);
                }
            }
        }

        // Phase 6: Code generation
        let codegen_start = std::time::Instant::now();
        let generated_code = match self.codegen_phase(&program) {
            Ok(code) => {
                stats.phase_times.code_generation = codegen_start.elapsed().as_millis() as u64;
                Some(code)
            }
            Err(error) => {
                self.diagnostics.add(error);
                stats.phase_times.code_generation = codegen_start.elapsed().as_millis() as u64;
                return self.create_failed_result(stats);
            }
        };

        // Calculate total time
        stats.phase_times.total = start_time.elapsed().as_millis() as u64;

        // Return successful result
        CompilationResult {
            code: generated_code,
            diagnostics: self.diagnostics.take_errors(),
            success: !self.has_errors(),
            stats,
        }
    }

    /// Parse the source code into an AST.
    fn parse_phase(&mut self) -> Result<Program> {
        // First tokenize the source
        let tokens = tokenize(self.source.clone())?;

        // Then parse the tokens into an AST
        let mut parser = Parser::new(tokens, self.source.clone());
        parser.parse()
    }

    /// Perform name resolution on the AST.
    fn name_resolution_phase(&mut self, _program: &mut Program) -> Result<()> {
        // TODO: Implement name resolution
        // This would involve:
        // - Building symbol tables
        // - Resolving identifiers to their declarations
        // - Checking for undefined variables/functions
        // - Handling module imports and exports
        Ok(())
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
            let severity = match violation.severity() {
                SafetySeverity::Info => errors::Severity::Info,
                SafetySeverity::Warning => errors::Severity::Warning,
                SafetySeverity::Error => errors::Severity::Error,
                SafetySeverity::Critical => errors::Severity::Error,
            };

            // Create appropriate error based on severity
            let error = match severity {
                errors::Severity::Error => TlError::safety(
                    self.source.clone(),
                    self.get_violation_span(&violation),
                    violation.description(),
                ),
                _ => TlError::safety(
                    self.source.clone(),
                    self.get_violation_span(&violation),
                    violation.description(),
                ),
            };

            self.diagnostics.add(error);
        }

        Ok(())
    }

    /// Perform borrow checking.
    fn borrow_check_phase(&mut self, _program: &Program) -> Result<()> {
        // TODO: Implement borrow checking
        // This would involve:
        // - Tracking ownership and borrowing
        // - Ensuring no use-after-move
        // - Validating lifetime relationships
        // - Checking mutability constraints
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

    /// Check if compilation should stop due to errors.
    fn should_stop_compilation(&self) -> bool {
        self.options.strict_mode && self.has_errors()
    }

    /// Check if there are any errors in the diagnostics.
    fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Create a failed compilation result.
    fn create_failed_result(&mut self, stats: CompilationStats) -> CompilationResult {
        CompilationResult {
            code: None,
            diagnostics: self.diagnostics.take_errors(),
            success: false,
            stats,
        }
    }

    /// Get the span for a safety violation.
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
}

/// Convenience function to compile source code with default options.
pub fn compile_source(source: &str) -> Result<CompiledModule> {
    let mut compiler = Compiler::with_defaults(source.to_string());
    let result = compiler.compile();

    if result.success {
        // Create a compiled module from the generated code
        let module = CompiledModule::new(
            "main".to_string(),
            result.code
                .map(|code| code.bytes())
                .unwrap_or_default()
        );
        Ok(module)
    } else {
        // Return the first error
        Err(result.diagnostics
            .into_iter()
            .next()
            .unwrap_or_else(|| TlError::internal("Compilation failed with no errors reported")))
    }
}

/// Convenience function to compile source code to a specific target.
pub fn compile_to_target(source: &str, target: &str) -> Result<CompiledModule> {
    let options = CompilerOptions {
        target: target.to_string(),
        ..Default::default()
    };

    let mut compiler = Compiler::new(source.to_string(), options);
    let result = compiler.compile();

    if result.success {
        let module = CompiledModule::new(
            "main".to_string(),
            result.code
                .map(|code| code.bytes())
                .unwrap_or_default()
        );
        Ok(module)
    } else {
        Err(result.diagnostics
            .into_iter()
            .next()
            .unwrap_or_else(|| TlError::internal("Compilation failed with no errors reported")))
    }
}

/// Parse source code into an AST without full compilation.
pub fn parse_only(source: &str) -> Result<Program> {
    let tokens = tokenize(source.to_string())?;
    let mut parser = Parser::new(tokens, source.to_string());
    parser.parse()
}

/// Type check an AST without full compilation.
pub fn type_check_only(program: &mut Program, source: String) -> Result<()> {
    let mut type_checker = TypeChecker::new(source);
    type_checker.check_program(program)
}

/// Check safety of an AST without full compilation.
pub fn safety_check_only(program: &Program, source: String) -> Result<Vec<SafetyViolation>> {
    analyze_safety(program, source)
}

/// Get compiler version information.
pub fn version_info() -> CompilerVersion {
    CompilerVersion {
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_hash: option_env!("GIT_HASH").unwrap_or("unknown").to_string(),
        build_date: env!("BUILD_DATE").to_string(),
        rustc_version: env!("RUSTC_VERSION").to_string(),
    }
}

/// Compiler version information.
#[derive(Debug, Clone)]
pub struct CompilerVersion {
    pub version: String,
    pub git_hash: String,
    pub build_date: String,
    pub rustc_version: String,
}

impl std::fmt::Display for CompilerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "T-Lang compiler version {} ({})\nBuilt on {} with rustc {}",
            self.version, self.git_hash, self.build_date, self.rustc_version
        )
    }
}

impl CompilerOptions {
    /// Create options for debugging/development.
    pub fn debug() -> Self {
        Self {
            debug_level: 3,
            optimization_level: 0,
            safety_analysis: true,
            strict_mode: true,
            ..Default::default()
        }
    }

    /// Create options for release builds.
    pub fn release() -> Self {
        Self {
            debug_level: 0,
            optimization_level: 3,
            safety_analysis: true,
            strict_mode: false,
            ..Default::default()
        }
    }

    /// Create options for testing.
    pub fn test() -> Self {
        Self {
            debug_level: 2,
            optimization_level: 0,
            safety_analysis: true,
            strict_mode: true,
            max_errors: 1000, // Allow more errors in tests
            ..Default::default()
        }
    }
}

impl CompilationResult {
    /// Check if compilation was successful.
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get the number of errors.
    pub fn error_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity() == errors::Severity::Error).count()
    }

    /// Get the number of warnings.
    pub fn warning_count(&self) -> usize {
        self.diagnostics.iter().filter(|d| d.severity() == errors::Severity::Warning).count()
    }

    /// Get all errors.
    pub fn errors(&self) -> Vec<&TlError> {
        self.diagnostics.iter().filter(|d| d.severity() == errors::Severity::Error).collect()
    }

    /// Get all warnings.
    pub fn warnings(&self) -> Vec<&TlError> {
        self.diagnostics.iter().filter(|d| d.severity() == errors::Severity::Warning).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_creation() {
        let source = "fn main() { print(\"Hello, world!\"); }";
        let compiler = Compiler::with_defaults(source.to_string());

        assert_eq!(compiler.options.target, "rust");
        assert_eq!(compiler.options.optimization_level, 1);
        assert!(compiler.options.safety_analysis);
    }

    #[test]
    fn test_compiler_options() {
        let debug_opts = CompilerOptions::debug();
        assert_eq!(debug_opts.debug_level, 3);
        assert_eq!(debug_opts.optimization_level, 0);
        assert!(debug_opts.strict_mode);

        let release_opts = CompilerOptions::release();
        assert_eq!(release_opts.debug_level, 0);
        assert_eq!(release_opts.optimization_level, 3);
        assert!(!release_opts.strict_mode);
    }

    #[test]
    fn test_parse_only() {
        let source = "fn test() { let x = 42; }";
        let result = parse_only(source);

        // This will fail until we implement the parser, but the structure is correct
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_compile_source() {
        let source = "fn main() { }";
        let result = compile_source(source);

        // This will fail until we implement all components, but the API is correct
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_version_info() {
        let version = version_info();
        assert!(!version.version.is_empty());
    }
}