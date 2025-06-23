// compiler/src/safety/analyzer.rs
//! Safety analysis for T-Lang safety-critical systems.
//!
//! Performs comprehensive safety checks including:
//! - Memory safety analysis
//! - Resource leak detection
//! - Timing analysis for real-time systems
//! - Null pointer analysis
//! - Buffer overflow detection

use shared::{
    Program, Item, ItemKind, Stmt, StmtKind, Expr, ExprKind, Type, TypeKind,
    SafetyLevel, Result, TlError
};
use miette::SourceSpan;
use std::collections::{HashMap, HashSet};

/// Safety analysis context and results.
pub struct SafetyAnalyzer {
    /// Source code for error reporting
    source: String,
    /// Currently active variables and their safety status
    variables: HashMap<String, VariableSafety>,
    /// Memory allocations that need to be freed
    pending_allocations: HashSet<AllocationId>,
    /// Resource acquisitions that need to be released
    pending_resources: HashSet<ResourceId>,
    /// Safety violations found during analysis
    violations: Vec<SafetyViolation>,
    /// Function call stack for recursion detection
    call_stack: Vec<String>,
    /// Maximum allowed call depth for real-time systems
    max_call_depth: usize,
}

/// Safety information about a variable.
#[derive(Debug, Clone)]
pub struct VariableSafety {
    /// Whether the variable has been initialized
    pub initialized: bool,
    /// Whether the variable has been moved (no longer accessible)
    pub moved: bool,
    /// Whether the variable is borrowed
    pub borrowed: bool,
    /// Lifetime information
    pub lifetime: Option<String>,
    /// Safety level required to access this variable
    pub safety_level: SafetyLevel,
}

/// Unique identifier for memory allocations.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct AllocationId(u64);

/// Unique identifier for system resources.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ResourceId(u64);

/// Types of safety violations.
#[derive(Debug, Clone)]
pub enum SafetyViolation {
    /// Use of uninitialized variable
    UninitializedVariable {
        name: String,
        span: SourceSpan,
    },
    /// Use after move
    UseAfterMove {
        name: String,
        span: SourceSpan,
        move_location: SourceSpan,
    },
    /// Memory leak - allocation without corresponding deallocation
    MemoryLeak {
        allocation_id: AllocationId,
        allocation_site: SourceSpan,
    },
    /// Resource leak - resource acquisition without release
    ResourceLeak {
        resource_id: ResourceId,
        resource_type: String,
        acquisition_site: SourceSpan,
    },
    /// Potential null pointer dereference
    NullPointerDereference {
        span: SourceSpan,
        expression: String,
    },
    /// Buffer overflow risk
    BufferOverflow {
        span: SourceSpan,
        buffer_size: Option<u64>,
        access_index: String,
    },
    /// Stack overflow risk from deep recursion
    StackOverflow {
        span: SourceSpan,
        call_depth: usize,
        function_name: String,
    },
    /// Unsafe operation in safe context
    UnsafeOperation {
        span: SourceSpan,
        operation: String,
        required_safety: SafetyLevel,
    },
    /// Data race potential
    DataRace {
        span: SourceSpan,
        variable: String,
        conflicting_access: SourceSpan,
    },
    /// Real-time constraint violation
    RealtimeViolation {
        span: SourceSpan,
        function: String,
        max_time: u64,
        estimated_time: u64,
    },
}

impl SafetyAnalyzer {
    /// Create a new safety analyzer.
    pub fn new(source: String) -> Self {
        Self {
            source,
            variables: HashMap::new(),
            pending_allocations: HashSet::new(),
            pending_resources: HashSet::new(),
            violations: Vec::new(),
            call_stack: Vec::new(),
            max_call_depth: 256, // Default stack limit for safety-critical systems
        }
    }

    /// Analyze a complete program for safety violations.
    pub fn analyze_program(&mut self, program: &Program) -> Result<Vec<SafetyViolation>> {
        self.violations.clear();

        // Analyze each top-level item
        for item in &program.items {
            self.analyze_item(item)?;
        }

        // Check for resource leaks at program end
        self.check_resource_leaks();

        Ok(self.violations.clone())
    }

    /// Analyze a top-level item.
    fn analyze_item(&mut self, item: &Item) -> Result<()> {
        match &item.kind {
            ItemKind::Function { name, body, safety, .. } => {
                self.analyze_function(name, body.as_ref(), *safety, item.span)?;
            }

            ItemKind::Static { value, .. } => {
                if let Some(init_expr) = Some(value) {
                    self.analyze_expr(init_expr)?;
                }
            }

            ItemKind::Const { value, .. } => {
                self.analyze_expr(value)?;
            }

            _ => {} // Other items don't contain executable code
        }

        Ok(())
    }

    /// Analyze a function for safety violations.
    fn analyze_function(&mut self, name: &str, body: Option<&Expr>,
                        safety_level: SafetyLevel, span: SourceSpan) -> Result<()> {
        // Check for stack overflow risk
        if self.call_stack.len() >= self.max_call_depth {
            self.violations.push(SafetyViolation::StackOverflow {
                span,
                call_depth: self.call_stack.len(),
                function_name: name.to_string(),
            });
            return Ok(()); // Don't continue analysis to prevent infinite recursion
        }

        // Enter function scope
        self.call_stack.push(name.to_string());
        let prev_variables = self.variables.clone();

        // Analyze function body if present
        if let Some(body_expr) = body {
            self.analyze_expr_in_context(body_expr, safety_level)?;
        }

        // Exit function scope
        self.variables = prev_variables;
        self.call_stack.pop();

        Ok(())
    }

    /// Analyze an expression for safety violations.
    fn analyze_expr(&mut self, expr: &Expr) -> Result<()> {
        self.analyze_expr_in_context(expr, SafetyLevel::Safe)
    }

    /// Analyze an expression within a specific safety context.
    fn analyze_expr_in_context(&mut self, expr: &Expr, context_safety: SafetyLevel) -> Result<()> {
        match &expr.kind {
            ExprKind::Variable { path } => {
                self.check_variable_access(path, expr.span)?;
            }

            ExprKind::Call { callee, args, safety } => {
                // Check if the call is safe in the current context
                if !self.is_safety_compatible(*safety, context_safety) {
                    self.violations.push(SafetyViolation::UnsafeOperation {
                        span: expr.span,
                        operation: "function call".to_string(),
                        required_safety: *safety,
                    });
                }

                // Analyze callee and arguments
                self.analyze_expr_in_context(callee, *safety)?;
                for arg in args {
                    self.analyze_expr_in_context(arg, *safety)?;
                }

                // Check for specific unsafe operations
                self.check_unsafe_call(callee, args, expr.span)?;
            }

            ExprKind::Binary { left, right, .. } => {
                self.analyze_expr_in_context(left, context_safety)?;
                self.analyze_expr_in_context(right, context_safety)?;
            }

            ExprKind::Unary { expr: inner, .. } => {
                self.analyze_expr_in_context(inner, context_safety)?;
            }

            ExprKind::Assign { target, value, .. } => {
                self.analyze_assignment(target, value, expr.span)?;
            }

            ExprKind::If { condition, then_branch, else_branch } => {
                self.analyze_expr_in_context(condition, context_safety)?;
                self.analyze_expr_in_context(then_branch, context_safety)?;
                if let Some(else_expr) = else_branch {
                    self.analyze_expr_in_context(else_expr, context_safety)?;
                }
            }

            ExprKind::Block(block) => {
                // Enter block scope
                let prev_variables = self.variables.clone();

                for stmt in &block.statements {
                    self.analyze_stmt_in_context(stmt, context_safety)?;
                }

                if let Some(block_expr) = &block.expr {
                    self.analyze_expr_in_context(block_expr, context_safety)?;
                }

                // Exit block scope, but preserve variable state changes
                self.merge_variable_states(prev_variables);
            }

            ExprKind::Index { object, index } => {
                self.analyze_expr_in_context(object, context_safety)?;
                self.analyze_expr_in_context(index, context_safety)?;
                self.check_buffer_access(object, index, expr.span)?;
            }

            ExprKind::Dereference { expr: target } => {
                self.analyze_expr_in_context(target, context_safety)?;
                self.check_null_dereference(target, expr.span)?;
            }

            ExprKind::Reference { expr: target, .. } => {
                self.analyze_expr_in_context(target, context_safety)?;
                self.check_borrow_rules(target, expr.span)?;
            }

            ExprKind::Unsafe { body } => {
                // Unsafe blocks allow unsafe operations
                self.analyze_expr_in_context(body, SafetyLevel::Unsafe)?;
            }

            _ => {
                // Handle other expression types as needed
            }
        }

        Ok(())
    }

    /// Analyze a statement for safety violations.
    fn analyze_stmt_in_context(&mut self, stmt: &Stmt, context_safety: SafetyLevel) -> Result<()> {
        match &stmt.kind {
            StmtKind::Expr(expr) => {
                self.analyze_expr_in_context(expr, context_safety)?;
            }

            StmtKind::Let { pattern, initializer, .. } => {
                if let shared::PatternKind::Ident(name) = &pattern.kind {
                    let var_safety = VariableSafety {
                        initialized: initializer.is_some(),
                        moved: false,
                        borrowed: false,
                        lifetime: None,
                        safety_level: context_safety,
                    };

                    self.variables.insert(name.clone(), var_safety);

                    if let Some(init_expr) = initializer {
                        self.analyze_expr_in_context(init_expr, context_safety)?;
                    }
                }
            }

            _ => {}
        }

        Ok(())
    }

    // Safety checking methods

    fn check_variable_access(&mut self, path: &[String], span: SourceSpan) -> Result<()> {
        if path.len() == 1 {
            let name = &path[0];
            if let Some(var_safety) = self.variables.get(name) {
                if !var_safety.initialized {
                    self.violations.push(SafetyViolation::UninitializedVariable {
                        name: name.clone(),
                        span,
                    });
                }

                if var_safety.moved {
                    self.violations.push(SafetyViolation::UseAfterMove {
                        name: name.clone(),
                        span,
                        move_location: span, // TODO: Track actual move location
                    });
                }
            }
        }

        Ok(())
    }

    fn check_unsafe_call(&mut self, callee: &Expr, args: &[Expr], span: SourceSpan) -> Result<()> {
        // Check for known unsafe functions
        if let ExprKind::Variable { path } = &callee.kind {
            if path.len() == 1 {
                let func_name = &path[0];
                match func_name.as_str() {
                    "malloc" | "alloc" | "allocate" => {
                        // Memory allocation - track for leak detection
                        let alloc_id = AllocationId(self.pending_allocations.len() as u64);
                        self.pending_allocations.insert(alloc_id);
                    }

                    "free" | "dealloc" | "deallocate" => {
                        // Memory deallocation - remove from pending
                        // TODO: Match with specific allocation
                    }

                    "unsafe_ptr_read" | "unsafe_ptr_write" => {
                        self.violations.push(SafetyViolation::UnsafeOperation {
                            span,
                            operation: format!("call to {}", func_name),
                            required_safety: SafetyLevel::Unsafe,
                        });
                    }

                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn check_buffer_access(&mut self, buffer: &Expr, index: &Expr, span: SourceSpan) -> Result<()> {
        // Static analysis for buffer bounds checking
        // This is a simplified version - a full implementation would need more sophisticated analysis

        if let Some(buffer_type) = &buffer.ty {
            if let TypeKind::Array { size, .. } = &buffer_type.kind {
                // Try to determine if index is within bounds
                if let ExprKind::Literal(shared::Literal::Integer(idx)) = &index.kind {
                    if let shared::types::ArraySize::Literal(size_val) = size {
                        if (*idx as u64) >= *size_val {
                            self.violations.push(SafetyViolation::BufferOverflow {
                                span,
                                buffer_size: Some(*size_val),
                                access_index: idx.to_string(),
                            });
                        }
                    }
                } else {
                    // Dynamic index - potential overflow
                    self.violations.push(SafetyViolation::BufferOverflow {
                        span,
                        buffer_size: None,
                        access_index: "dynamic".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn check_null_dereference(&mut self, target: &Expr, span: SourceSpan) -> Result<()> {
        // Check for potential null pointer dereference
        // This would need flow analysis to be fully effective

        if let Some(target_type) = &target.ty {
            if let TypeKind::Pointer { .. } = &target_type.kind {
                // Pointer dereference - could be null
                self.violations.push(SafetyViolation::NullPointerDereference {
                    span,
                    expression: format!("{:?}", target.kind),
                });
            }
        }

        Ok(())
    }

    fn check_borrow_rules(&mut self, target: &Expr, span: SourceSpan) -> Result<()> {
        // Check Rust-style borrowing rules
        if let ExprKind::Variable { path } = &target.kind {
            if path.len() == 1 {
                let name = &path[0];
                if let Some(var_safety) = self.variables.get_mut(name) {
                    if var_safety.borrowed {
                        // Multiple borrows - potential data race
                        self.violations.push(SafetyViolation::DataRace {
                            span,
                            variable: name.clone(),
                            conflicting_access: span,
                        });
                    } else {
                        var_safety.borrowed = true;
                    }
                }
            }
        }

        Ok(())
    }

    fn analyze_assignment(&mut self, target: &Expr, value: &Expr, span: SourceSpan) -> Result<()> {
        // Analyze the value being assigned
        self.analyze_expr(value)?;

        // Update variable state for assignment target
        if let ExprKind::Variable { path } = &target.kind {
            if path.len() == 1 {
                let name = &path[0];
                if let Some(var_safety) = self.variables.get_mut(name) {
                    var_safety.initialized = true;
                    var_safety.moved = false; // Assignment reinitializes
                }
            }
        }

        Ok(())
    }

    // Helper methods

    fn is_safety_compatible(&self, required: SafetyLevel, context: SafetyLevel) -> bool {
        match (required, context) {
            (SafetyLevel::Safe, _) => true,
            (SafetyLevel::Unsafe, SafetyLevel::Unsafe) => true,
            (SafetyLevel::Critical, SafetyLevel::Critical) => true,
            _ => false,
        }
    }

    fn merge_variable_states(&mut self, prev_variables: HashMap<String, VariableSafety>) {
        // Merge variable states from block scope back to parent scope
        // Keep initialization and move status, but reset borrowing
        for (name, mut new_state) in self.variables.clone() {
            if let Some(prev_state) = prev_variables.get(&name) {
                new_state.borrowed = prev_state.borrowed;
                self.variables.insert(name, new_state);
            }
        }
    }

    fn check_resource_leaks(&mut self) {
        // Check for any remaining allocations or resources at program end
        for &alloc_id in &self.pending_allocations {
            self.violations.push(SafetyViolation::MemoryLeak {
                allocation_id: alloc_id,
                allocation_site: SourceSpan::new(0.into(), 0), // TODO: Track actual site
            });
        }

        for &resource_id in &self.pending_resources {
            self.violations.push(SafetyViolation::ResourceLeak {
                resource_id,
                resource_type: "unknown".to_string(), // TODO: Track resource type
                acquisition_site: SourceSpan::new(0.into(), 0),
            });
        }
    }
}

impl SafetyViolation {
    /// Get the severity level of this safety violation.
    pub fn severity(&self) -> SafetySeverity {
        match self {
            SafetyViolation::UninitializedVariable { .. } => SafetySeverity::Error,
            SafetyViolation::UseAfterMove { .. } => SafetySeverity::Error,
            SafetyViolation::MemoryLeak { .. } => SafetySeverity::Warning,
            SafetyViolation::ResourceLeak { .. } => SafetySeverity::Warning,
            SafetyViolation::NullPointerDereference { .. } => SafetySeverity::Critical,
            SafetyViolation::BufferOverflow { .. } => SafetySeverity::Critical,
            SafetyViolation::StackOverflow { .. } => SafetySeverity::Critical,
            SafetyViolation::UnsafeOperation { .. } => SafetySeverity::Error,
            SafetyViolation::DataRace { .. } => SafetySeverity::Critical,
            SafetyViolation::RealtimeViolation { .. } => SafetySeverity::Error,
        }
    }

    /// Get a human-readable description of this violation.
    pub fn description(&self) -> String {
        match self {
            SafetyViolation::UninitializedVariable { name, .. } => {
                format!("Use of uninitialized variable '{}'", name)
            }
            SafetyViolation::UseAfterMove { name, .. } => {
                format!("Use of variable '{}' after it was moved", name)
            }
            SafetyViolation::MemoryLeak { .. } => {
                "Memory leak: allocated memory was not freed".to_string()
            }
            SafetyViolation::ResourceLeak { resource_type, .. } => {
                format!("Resource leak: {} was not properly released", resource_type)
            }
            SafetyViolation::NullPointerDereference { .. } => {
                "Potential null pointer dereference".to_string()
            }
            SafetyViolation::BufferOverflow { .. } => {
                "Potential buffer overflow".to_string()
            }
            SafetyViolation::StackOverflow { function_name, call_depth, .. } => {
                format!("Stack overflow risk in function '{}' at depth {}", function_name, call_depth)
            }
            SafetyViolation::UnsafeOperation { operation, .. } => {
                format!("Unsafe operation '{}' used in safe context", operation)
            }
            SafetyViolation::DataRace { variable, .. } => {
                format!("Potential data race on variable '{}'", variable)
            }
            SafetyViolation::RealtimeViolation { function, max_time, estimated_time, .. } => {
                format!("Real-time constraint violation in '{}': {}ms > {}ms",
                        function, estimated_time, max_time)
            }
        }
    }
}

/// Severity levels for safety violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SafetySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Analyze a program for safety violations.
pub fn analyze_safety(program: &Program, source: String) -> Result<Vec<SafetyViolation>> {
    let mut analyzer = SafetyAnalyzer::new(source);
    analyzer.analyze_program(program)
}