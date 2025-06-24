// compiler/src/safety/mod.rs
//! Safety analysis for T-Lang.
//! Detects potential memory safety violations, resource leaks, and other safety issues.

use shared::{Program, Item, ItemKind, Expr, ExprKind, Stmt, StmtKind, Pattern, PatternKind, Result, TlError, HasSpan};
use miette::SourceSpan;
use std::collections::{HashMap, HashSet};

/// Safety analyzer for T-Lang programs.
pub struct SafetyAnalyzer {
    source: String,
    violations: Vec<SafetyViolation>,
    // Track variable states
    variables: HashMap<String, VariableState>,
    // Track resource acquisition and release
    resources: HashMap<String, ResourceState>,
    // Current scope depth
    scope_depth: usize,
    // Stack of scopes for proper cleanup
    scope_stack: Vec<Scope>,
    // Function call stack for recursion detection
    call_stack: Vec<String>,
    // Maximum call stack depth before warning
    max_call_depth: usize,
}

/// State of a variable for safety analysis.
#[derive(Debug, Clone)]
struct VariableState {
    /// Whether the variable has been initialized
    initialized: bool,
    /// Whether the variable has been moved
    moved: bool,
    /// Whether the variable is borrowed mutably
    borrowed_mut: bool,
    /// Whether the variable is borrowed immutably
    borrowed_immut: bool,
    /// Scope where the variable was declared
    scope: usize,
    /// Source location of declaration
    declaration_site: SourceSpan,
}

/// State of a resource for leak detection.
#[derive(Debug, Clone)]
struct ResourceState {
    /// Type of resource (file, memory, network, etc.)
    resource_type: ResourceType,
    /// Whether the resource has been released
    released: bool,
    /// Source location where resource was acquired
    acquisition_site: SourceSpan,
    /// Scope where the resource was acquired
    scope: usize,
}

/// Types of resources that need explicit cleanup.
#[derive(Debug, Clone, PartialEq)]
enum ResourceType {
    Memory,
    File,
    Network,
    Database,
    Mutex,
    Thread,
    Other(String),
}

/// A scope in the safety analysis.
#[derive(Debug, Clone)]
struct Scope {
    /// Variables declared in this scope
    variables: HashSet<String>,
    /// Resources acquired in this scope
    resources: HashSet<String>,
    /// Depth of this scope
    depth: usize,
}

/// Safety violations detected during analysis.
#[derive(Debug, Clone)]
pub enum SafetyViolation {
    /// Use of uninitialized variable
    UninitializedVariable {
        span: SourceSpan,
        name: String,
    },

    /// Use of variable after it has been moved
    UseAfterMove {
        span: SourceSpan,
        var: String,
        move_site: SourceSpan,
    },

    /// Memory leak (allocated but never freed)
    MemoryLeak {
        allocation_site: SourceSpan,
        resource_type: String,
    },

    /// Resource leak (acquired but never released)
    ResourceLeak {
        acquisition_site: SourceSpan,
        resource_type: String,
    },

    /// Potential null pointer dereference
    NullPointerDereference {
        span: SourceSpan,
        pointer: String,
    },

    /// Buffer overflow (array access out of bounds)
    BufferOverflow {
        span: SourceSpan,
        array: String,
        index: String,
    },

    /// Stack overflow (excessive recursion)
    StackOverflow {
        span: SourceSpan,
        function: String,
        depth: usize,
    },

    /// Unsafe operation in safe context
    UnsafeOperation {
        span: SourceSpan,
        operation: String,
    },

    /// Data race (concurrent access without synchronization)
    DataRace {
        span: SourceSpan,
        variable: String,
    },

    /// Real-time constraint violation
    RealtimeViolation {
        span: SourceSpan,
        operation: String,
        reason: String,
    },
}

/// Severity levels for safety violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl SafetyViolation {
    /// Get the severity of this safety violation.
    pub fn severity(&self) -> SafetySeverity {
        match self {
            SafetyViolation::UninitializedVariable { .. } => SafetySeverity::Error,
            SafetyViolation::UseAfterMove { .. } => SafetySeverity::Error,
            SafetyViolation::MemoryLeak { .. } => SafetySeverity::Warning,
            SafetyViolation::ResourceLeak { .. } => SafetySeverity::Warning,
            SafetyViolation::NullPointerDereference { .. } => SafetySeverity::Critical,
            SafetyViolation::BufferOverflow { .. } => SafetySeverity::Critical,
            SafetyViolation::StackOverflow { .. } => SafetySeverity::Error,
            SafetyViolation::UnsafeOperation { .. } => SafetySeverity::Warning,
            SafetyViolation::DataRace { .. } => SafetySeverity::Critical,
            SafetyViolation::RealtimeViolation { .. } => SafetySeverity::Warning,
        }
    }

    /// Get a human-readable description of this violation.
    pub fn description(&self) -> String {
        match self {
            SafetyViolation::UninitializedVariable { name, .. } => {
                format!("Use of uninitialized variable '{}'", name)
            }
            SafetyViolation::UseAfterMove { var, .. } => {
                format!("Use of variable '{}' after it has been moved", var)
            }
            SafetyViolation::MemoryLeak { resource_type, .. } => {
                format!("Memory leak: {} was allocated but never freed", resource_type)
            }
            SafetyViolation::ResourceLeak { resource_type, .. } => {
                format!("Resource leak: {} was acquired but never released", resource_type)
            }
            SafetyViolation::NullPointerDereference { pointer, .. } => {
                format!("Potential null pointer dereference of '{}'", pointer)
            }
            SafetyViolation::BufferOverflow { array, index, .. } => {
                format!("Potential buffer overflow: array '{}' accessed with index '{}'", array, index)
            }
            SafetyViolation::StackOverflow { function, depth, .. } => {
                format!("Potential stack overflow: function '{}' called recursively {} times", function, depth)
            }
            SafetyViolation::UnsafeOperation { operation, .. } => {
                format!("Unsafe operation '{}' used in safe context", operation)
            }
            SafetyViolation::DataRace { variable, .. } => {
                format!("Potential data race on variable '{}'", variable)
            }
            SafetyViolation::RealtimeViolation { operation, reason, .. } => {
                format!("Real-time constraint violation: {} ({})", operation, reason)
            }
        }
    }

    /// Get the source span for this violation.
    pub fn span(&self) -> SourceSpan {
        match self {
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

impl SafetyAnalyzer {
    /// Create a new safety analyzer.
    pub fn new(source: String) -> Self {
        Self {
            source,
            violations: Vec::new(),
            variables: HashMap::new(),
            resources: HashMap::new(),
            scope_depth: 0,
            scope_stack: vec![Scope {
                variables: HashSet::new(),
                resources: HashSet::new(),
                depth: 0,
            }],
            call_stack: Vec::new(),
            max_call_depth: 1000, // Configurable recursion limit
        }
    }

    /// Analyze a program for safety violations.
    pub fn analyze(&mut self, program: &Program) -> Vec<SafetyViolation> {
        self.violations.clear();

        // Analyze all top-level items
        for item in &program.items {
            self.analyze_item(item);
        }

        // Check for resource leaks at program end
        self.check_resource_leaks();

        std::mem::take(&mut self.violations)
    }

    /// Analyze a top-level item.
    fn analyze_item(&mut self, item: &Item) {
        match &item.kind {
            ItemKind::Function { name, body, .. } => {
                self.analyze_function(name, body);
            }
            ItemKind::Const { value, .. } => {
                self.analyze_expr(value);
            }
            ItemKind::Static { value, .. } => {
                self.analyze_expr(value);
            }
            ItemKind::Module { items, .. } => {
                self.enter_scope();
                for item in items {
                    self.analyze_item(item);
                }
                self.exit_scope();
            }
            // Other items don't contain executable code
            _ => {}
        }
    }

    /// Analyze a function for safety violations.
    fn analyze_function(&mut self, name: &str, body: &Expr) {
        // Check for recursion
        if self.call_stack.contains(&name.to_string()) {
            let depth = self.call_stack.len();
            if depth > self.max_call_depth {
                self.violations.push(SafetyViolation::StackOverflow {
                    span: body.span(),
                    function: name.to_string(),
                    depth,
                });
            }
        }

        self.call_stack.push(name.to_string());
        self.enter_scope();

        self.analyze_expr(body);

        self.exit_scope();
        self.call_stack.pop();
    }

    /// Analyze an expression for safety violations.
    fn analyze_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Identifier(name) => {
                self.check_variable_use(name, expr.span());
            }

            ExprKind::Binary { lhs, rhs, .. } => {
                self.analyze_expr(lhs);
                self.analyze_expr(rhs);
            }

            ExprKind::Unary { operand, .. } => {
                self.analyze_expr(operand);
            }

            ExprKind::Call { func, args } => {
                self.analyze_expr(func);
                for arg in args {
                    self.analyze_expr(arg);
                }

                // Check for resource operations
                if let ExprKind::Identifier(func_name) = &func.kind {
                    self.check_resource_operation(func_name, args, expr.span());
                }
            }

            ExprKind::FieldAccess { object, .. } => {
                self.analyze_expr(object);
            }

            ExprKind::Index { object, index } => {
                self.analyze_expr(object);
                self.analyze_expr(index);

                // Check for potential buffer overflow
                if let ExprKind::Identifier(array_name) = &object.kind {
                    if let ExprKind::Identifier(index_name) = &index.kind {
                        self.check_buffer_access(array_name, index_name, expr.span());
                    }
                }
            }

            ExprKind::Assignment { target, value, .. } => {
                self.analyze_expr(value);

                // Handle assignment to variables
                if let ExprKind::Identifier(var_name) = &target.kind {
                    self.assign_variable(var_name, expr.span());
                } else {
                    self.analyze_expr(target);
                }
            }

            ExprKind::Block { stmts, expr: block_expr } => {
                self.enter_scope();

                for stmt in stmts {
                    self.analyze_stmt(stmt);
                }

                if let Some(expr) = block_expr {
                    self.analyze_expr(expr);
                }

                self.exit_scope();
            }

            ExprKind::If { condition, then_branch, else_branch } => {
                self.analyze_expr(condition);
                self.analyze_expr(then_branch);
                if let Some(else_expr) = else_branch {
                    self.analyze_expr(else_expr);
                }
            }

            ExprKind::While { condition, body } => {
                self.analyze_expr(condition);
                self.analyze_expr(body);
            }

            ExprKind::For { pattern, iterable, body } => {
                self.analyze_expr(iterable);
                self.enter_scope();
                self.declare_pattern_variables(pattern, expr.span());
                self.analyze_expr(body);
                self.exit_scope();
            }

            ExprKind::Match { expr: match_expr, arms } => {
                self.analyze_expr(match_expr);
                for arm in arms {
                    self.enter_scope();
                    self.declare_pattern_variables(&arm.pattern, arm.span);
                    if let Some(guard) = &arm.guard {
                        self.analyze_expr(guard);
                    }
                    self.analyze_expr(&arm.body);
                    self.exit_scope();
                }
            }

            ExprKind::Reference { expr: ref_expr, .. } => {
                self.analyze_expr(ref_expr);

                // Mark variable as borrowed if it's an identifier
                if let ExprKind::Identifier(var_name) = &ref_expr.kind {
                    self.borrow_variable(var_name, false, expr.span());
                }
            }

            ExprKind::Dereference { expr: deref_expr } => {
                self.analyze_expr(deref_expr);

                // Check for potential null pointer dereference
                if let ExprKind::Identifier(ptr_name) = &deref_expr.kind {
                    self.check_null_dereference(ptr_name, expr.span());
                }
            }

            ExprKind::Unsafe { body } => {
                // Unsafe blocks require special analysis
                self.analyze_expr(body);
            }

            ExprKind::Tuple(exprs) => {
                for expr in exprs {
                    self.analyze_expr(expr);
                }
            }

            ExprKind::Array { elements, repeat } => {
                for expr in elements {
                    self.analyze_expr(expr);
                }
                if let Some(repeat_expr) = repeat {
                    self.analyze_expr(repeat_expr);
                }
            }

            ExprKind::Struct { fields, base, .. } => {
                for field in fields {
                    self.analyze_expr(&field.value);
                }
                if let Some(base_expr) = base {
                    self.analyze_expr(base_expr);
                }
            }

            ExprKind::Closure { body, .. } => {
                self.enter_scope();
                self.analyze_expr(body);
                self.exit_scope();
            }

            // Literals and other simple expressions don't need special analysis
            _ => {}
        }
    }

    /// Analyze a statement for safety violations.
    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let { pattern, initializer, .. } => {
                if let Some(init_expr) = initializer {
                    self.analyze_expr(init_expr);
                }
                self.declare_pattern_variables(pattern, stmt.span());
            }

            StmtKind::Expr(expr) => {
                self.analyze_expr(expr);
            }

            StmtKind::Item(item) => {
                self.analyze_item(item);
            }

            StmtKind::Macro { .. } => {
                // Macro analysis would require expansion first
            }
        }
    }

    /// Declare variables from a pattern.
    fn declare_pattern_variables(&mut self, pattern: &Pattern, span: SourceSpan) {
        match &pattern.kind {
            PatternKind::Identifier { name, .. } => {
                self.declare_variable(name, span);
            }

            PatternKind::Tuple(patterns) => {
                for pat in patterns {
                    self.declare_pattern_variables(pat, span);
                }
            }

            PatternKind::Struct { fields, .. } => {
                for (_, pat) in fields {
                    self.declare_pattern_variables(pat, span);
                }
            }

            PatternKind::Or(patterns) => {
                // All patterns in an OR must bind the same variables
                if let Some(first) = patterns.first() {
                    self.declare_pattern_variables(first, span);
                }
            }

            PatternKind::Ref(inner) => {
                self.declare_pattern_variables(inner, span);
            }

            // Other patterns don't bind variables
            _ => {}
        }
    }

    /// Declare a new variable.
    fn declare_variable(&mut self, name: &str, span: SourceSpan) {
        let state = VariableState {
            initialized: true, // Assume initialized if there's a pattern
            moved: false,
            borrowed_mut: false,
            borrowed_immut: false,
            scope: self.scope_depth,
            declaration_site: span,
        };

        self.variables.insert(name.to_string(), state);

        if let Some(current_scope) = self.scope_stack.last_mut() {
            current_scope.variables.insert(name.to_string());
        }
    }

    /// Assign to a variable.
    fn assign_variable(&mut self, name: &str, _span: SourceSpan) {
        if let Some(state) = self.variables.get_mut(name) {
            state.initialized = true;
            state.moved = false; // Assignment re-initializes
        }
    }

    /// Check usage of a variable.
    fn check_variable_use(&mut self, name: &str, span: SourceSpan) {
        if let Some(state) = self.variables.get(name) {
            if !state.initialized {
                self.violations.push(SafetyViolation::UninitializedVariable {
                    span,
                    name: name.to_string(),
                });
            }

            if state.moved {
                self.violations.push(SafetyViolation::UseAfterMove {
                    span,
                    var: name.to_string(),
                    move_site: state.declaration_site,
                });
            }
        }
    }

    /// Borrow a variable.
    fn borrow_variable(&mut self, name: &str, mutable: bool, _span: SourceSpan) {
        if let Some(state) = self.variables.get_mut(name) {
            if mutable {
                state.borrowed_mut = true;
            } else {
                state.borrowed_immut = true;
            }
        }
    }

    /// Check for resource operations (allocation, deallocation, etc.).
    fn check_resource_operation(&mut self, func_name: &str, _args: &[Expr], span: SourceSpan) {
        match func_name {
            "malloc" | "alloc" | "new" => {
                // Resource allocation
                let resource_id = format!("{}_{}", func_name, span.offset());
                let state = ResourceState {
                    resource_type: ResourceType::Memory,
                    released: false,
                    acquisition_site: span,
                    scope: self.scope_depth,
                };
                self.resources.insert(resource_id.clone(), state);

                if let Some(current_scope) = self.scope_stack.last_mut() {
                    current_scope.resources.insert(resource_id);
                }
            }

            "free" | "delete" | "close" | "release" => {
                // Resource deallocation - would need to track which resource
                // This is simplified; real implementation would need data flow analysis
            }

            "unsafe_operation" => {
                self.violations.push(SafetyViolation::UnsafeOperation {
                    span,
                    operation: func_name.to_string(),
                });
            }

            _ => {}
        }
    }

    /// Check for potential buffer overflow.
    fn check_buffer_access(&mut self, array_name: &str, index_name: &str, span: SourceSpan) {
        // This is a simplified check; real implementation would need more sophisticated analysis
        if index_name.contains("user_input") || index_name.contains("untrusted") {
            self.violations.push(SafetyViolation::BufferOverflow {
                span,
                array: array_name.to_string(),
                index: index_name.to_string(),
            });
        }
    }

    /// Check for potential null pointer dereference.
    fn check_null_dereference(&mut self, ptr_name: &str, span: SourceSpan) {
        // This is a simplified check; real implementation would need null analysis
        if ptr_name.contains("nullable") || ptr_name.contains("optional") {
            self.violations.push(SafetyViolation::NullPointerDereference {
                span,
                pointer: ptr_name.to_string(),
            });
        }
    }

    /// Enter a new scope.
    fn enter_scope(&mut self) {
        self.scope_depth += 1;
        self.scope_stack.push(Scope {
            variables: HashSet::new(),
            resources: HashSet::new(),
            depth: self.scope_depth,
        });
    }

    /// Exit the current scope.
    fn exit_scope(&mut self) {
        if let Some(scope) = self.scope_stack.pop() {
            // Check for resources that weren't released in this scope
            for resource_id in &scope.resources {
                if let Some(state) = self.resources.get(resource_id) {
                    if !state.released {
                        let resource_type = match state.resource_type {
                            ResourceType::Memory => "memory",
                            ResourceType::File => "file",
                            ResourceType::Network => "network connection",
                            ResourceType::Database => "database connection",
                            ResourceType::Mutex => "mutex",
                            ResourceType::Thread => "thread",
                            ResourceType::Other(ref name) => name,
                        };

                        self.violations.push(SafetyViolation::ResourceLeak {
                            acquisition_site: state.acquisition_site,
                            resource_type: resource_type.to_string(),
                        });
                    }
                }
            }

            // Remove variables that are going out of scope
            for var_name in &scope.variables {
                self.variables.remove(var_name);
            }

            // Remove resources that are going out of scope
            for resource_id in &scope.resources {
                self.resources.remove(resource_id);
            }
        }

        if self.scope_depth > 0 {
            self.scope_depth -= 1;
        }
    }

    /// Check for resource leaks at the end of analysis.
    fn check_resource_leaks(&mut self) {
        for (_, state) in &self.resources {
            if !state.released {
                let resource_type = match state.resource_type {
                    ResourceType::Memory => "memory",
                    ResourceType::File => "file",
                    ResourceType::Network => "network connection",
                    ResourceType::Database => "database connection",
                    ResourceType::Mutex => "mutex",
                    ResourceType::Thread => "thread",
                    ResourceType::Other(ref name) => name,
                };

                self.violations.push(SafetyViolation::ResourceLeak {
                    acquisition_site: state.acquisition_site,
                    resource_type: resource_type.to_string(),
                });
            }
        }
    }
}

/// Analyze a program for safety violations.
pub fn analyze_safety(program: &Program, source: String) -> Result<Vec<SafetyViolation>> {
    let mut analyzer = SafetyAnalyzer::new(source);
    Ok(analyzer.analyze(program))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::ast::*;

    fn dummy_span() -> SourceSpan {
        SourceSpan::new(0.into(), 0)
    }

    #[test]
    fn test_safety_analyzer_creation() {
        let analyzer = SafetyAnalyzer::new("test source".to_string());
        assert_eq!(analyzer.scope_depth, 0);
        assert!(analyzer.violations.is_empty());
    }

    #[test]
    fn test_uninitialized_variable_detection() {
        let mut analyzer = SafetyAnalyzer::new("let x; use(x);".to_string());

        // Simulate using an uninitialized variable
        let expr = Expr::identifier("x".to_string(), dummy_span());
        analyzer.check_variable_use("x", expr.span());

        assert!(!analyzer.violations.is_empty());
        if let SafetyViolation::UninitializedVariable { name, .. } = &analyzer.violations[0] {
            assert_eq!(name, "x");
        }
    }

    #[test]
    fn test_scope_management() {
        let mut analyzer = SafetyAnalyzer::new("test".to_string());

        assert_eq!(analyzer.scope_depth, 0);

        analyzer.enter_scope();
        assert_eq!(analyzer.scope_depth, 1);

        analyzer.exit_scope();
        assert_eq!(analyzer.scope_depth, 0);
    }

    #[test]
    fn test_violation_severity() {
        let violation = SafetyViolation::NullPointerDereference {
            span: dummy_span(),
            pointer: "ptr".to_string(),
        };

        assert_eq!(violation.severity(), SafetySeverity::Critical);
        assert!(violation.description().contains("null pointer"));
    }
}