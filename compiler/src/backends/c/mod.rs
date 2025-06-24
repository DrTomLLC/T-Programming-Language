//! C Backend for T-Lang Compiler
//!
//! This backend generates C code from TIR for maximum portability and compatibility.

use plugin_api::{Backend, BackendResult, CompiledArtifact};
use shared::tir::{TirModule, TirFunction, TirType, BasicBlock, Instruction, Opcode, Terminator, ConstantValue, ValueId, BlockId};
use errors::CompileError;
use std::collections::HashMap;
use std::fmt::Write;

/// C code generation backend
pub struct CBackend {
    name: String,
}

impl Default for CBackend {
    fn default() -> Self {
        Self {
            name: "c".to_string(),
        }
    }
}

impl Backend for CBackend {
    fn name(&self) -> &str {
        &self.name
    }

    fn compile(&self, module: TirModule) -> BackendResult {
        let mut generator = CCodeGenerator::new();
        let c_code = generator.generate_code(module)?;

        Ok(Box::new(CompiledArtifact {
            target: "c".to_string(),
            format: "source".to_string(),
            data: c_code.into_bytes(),
        }))
    }
}

/// C code generator implementation
struct CCodeGenerator {
    output: String,
    indent_level: usize,
    value_names: HashMap<ValueId, String>,
    block_labels: HashMap<BlockId, String>,
    value_counter: u32,
}

impl CCodeGenerator {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            value_names: HashMap::new(),
            block_labels: HashMap::new(),
            value_counter: 0,
        }
    }

    fn generate_code(&mut self, module: TirModule) -> Result<String, CompileError> {
        self.write_includes();
        self.write_forward_declarations(&module);

        // Generate struct/enum definitions
        self.write_type_definitions(&module);

        // Generate function implementations
        for function in module.functions.values() {
            self.generate_function(function)?;
        }

        Ok(self.output.clone())
    }

    fn write_includes(&mut self) {
        writeln!(self.output, "#include <stdio.h>").unwrap();
        writeln!(self.output, "#include <stdlib.h>").unwrap();
        writeln!(self.output, "#include <stdint.h>").unwrap();
        writeln!(self.output, "#include <stdbool.h>").unwrap();
        writeln!(self.output).unwrap();
    }

    fn write_forward_declarations(&mut self, module: &TirModule) {
        writeln!(self.output, "// Forward declarations").unwrap();
        for function in module.functions.values() {
            let return_type = self.tir_type_to_c(&function.signature.return_type);
            write!(self.output, "{} {}(", return_type, function.name).unwrap();

            for (i, (param_name, param_type)) in function.signature.parameters.iter().enumerate() {
                if i > 0 {
                    write!(self.output, ", ").unwrap();
                }
                write!(self.output, "{} {}", self.tir_type_to_c(param_type), param_name).unwrap();
            }

            if function.signature.parameters.is_empty() {
                write!(self.output, "void").unwrap();
            }

            writeln!(self.output, ");").unwrap();
        }
        writeln!(self.output).unwrap();
    }

    fn write_type_definitions(&mut self, module: &TirModule) {
        writeln!(self.output, "// Type definitions").unwrap();
        for (name, tir_type) in &module.types {
            match tir_type {
                TirType::Struct(_) => {
                    writeln!(self.output, "typedef struct {} {{", name).unwrap();
                    writeln!(self.output, "    // TODO: Add struct fields").unwrap();
                    writeln!(self.output, "}} {};", name).unwrap();
                }
                TirType::Enum(_) => {
                    writeln!(self.output, "typedef enum {{").unwrap();
                    writeln!(self.output, "    // TODO: Add enum variants").unwrap();
                    writeln!(self.output, "}} {};", name).unwrap();
                }
                _ => {}
            }
        }
        writeln!(self.output).unwrap();
    }

    fn generate_function(&mut self, function: &TirFunction) -> Result<(), CompileError> {
        self.value_names.clear();
        self.block_labels.clear();
        self.value_counter = 0;

        // Generate function signature
        let return_type = self.tir_type_to_c(&function.signature.return_type);
        write!(self.output, "{} {}(", return_type, function.name).unwrap();

        // Add parameters to value names
        for (i, (param_name, param_type)) in function.signature.parameters.iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ").unwrap();
            }
            write!(self.output, "{} {}", self.tir_type_to_c(param_type), param_name).unwrap();

            // For TIR lowering, parameters might have value IDs we need to track
            // This is simplified - in reality we'd need proper parameter mapping
        }

        if function.signature.parameters.is_empty() {
            write!(self.output, "void").unwrap();
        }

        writeln!(self.output, ") {{").unwrap();
        self.indent_level += 1;

        // Pre-generate block labels
        for block_id in function.blocks.keys() {
            let label = format!("block_{}", block_id.0);
            self.block_labels.insert(*block_id, label);
        }

        // Generate local variable declarations
        self.generate_local_declarations(function)?;

        // Generate function body
        if let Some(entry_block) = function.blocks.get(&function.entry_block) {
            self.generate_block(entry_block, function)?;
        }

        // Generate other blocks
        for (block_id, block) in &function.blocks {
            if *block_id != function.entry_block {
                writeln!(self.output).unwrap();
                self.write_indent();
                writeln!(self.output, "{}:", self.block_labels[block_id]).unwrap();
                self.generate_block(block, function)?;
            }
        }

        self.indent_level -= 1;
        writeln!(self.output, "}}").unwrap();
        writeln!(self.output).unwrap();

        Ok(())
    }

    fn generate_local_declarations(&mut self, function: &TirFunction) -> Result<(), CompileError> {
        // Collect all value IDs that need local variables
        let mut declared_values = std::collections::HashSet::new();

        for block in function.blocks.values() {
            for instruction in &block.instructions {
                if let Some(result_id) = instruction.result {
                    if !declared_values.contains(&result_id) {
                        declared_values.insert(result_id);
                        let var_name = self.get_value_name(result_id);
                        let c_type = self.tir_type_to_c(&instruction.ty);

                        self.write_indent();
                        writeln!(self.output, "{} {};", c_type, var_name).unwrap();
                    }
                }
            }
        }

        if !declared_values.is_empty() {
            writeln!(self.output).unwrap();
        }

        Ok(())
    }

    fn generate_block(&mut self, block: &BasicBlock, function: &TirFunction) -> Result<(), CompileError> {
        // Generate instructions
        for instruction in &block.instructions {
            self.generate_instruction(instruction)?;
        }

        // Generate terminator
        if let Some(ref terminator) = block.terminator {
            self.generate_terminator(terminator)?;
        }

        Ok(())
    }

    fn generate_instruction(&mut self, instruction: &Instruction) -> Result<(), CompileError> {
        self.write_indent();

        match &instruction.opcode {
            Opcode::Const(value) => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let c_value = self.constant_to_c(value);
                    writeln!(self.output, "{} = {};", var_name, c_value).unwrap();
                }
            }
            Opcode::Add => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} + {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Sub => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} - {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Mul => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} * {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Div => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} / {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Eq => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} == {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Ne => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} != {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Lt => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} < {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Le => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} <= {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Gt => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} > {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Ge => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} >= {};", var_name, left, right).unwrap();
                }
            }
            Opcode::And => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} && {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Or => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let left = self.get_value_name(instruction.operands[0]);
                    let right = self.get_value_name(instruction.operands[1]);
                    writeln!(self.output, "{} = {} || {};", var_name, left, right).unwrap();
                }
            }
            Opcode::Not => {
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    let operand = self.get_value_name(instruction.operands[0]);
                    writeln!(self.output, "{} = !{};", var_name, operand).unwrap();
                }
            }
            Opcode::Call => {
                // Simplified function call handling
                if let Some(result_id) = instruction.result {
                    let var_name = self.get_value_name(result_id);
                    write!(self.output, "{} = function_call(", var_name).unwrap();
                    for (i, &operand_id) in instruction.operands.iter().enumerate() {
                        if i > 0 {
                            write!(self.output, ", ").unwrap();
                        }
                        write!(self.output, "{}", self.get_value_name(operand_id)).unwrap();
                    }
                    writeln!(self.output, ");").unwrap();
                } else {
                    write!(self.output, "function_call(").unwrap();
                    for (i, &operand_id) in instruction.operands.iter().enumerate() {
                        if i > 0 {
                            write!(self.output, ", ").unwrap();
                        }
                        write!(self.output, "{}", self.get_value_name(operand_id)).unwrap();
                    }
                    writeln!(self.output, ");").unwrap();
                }
            }
            _ => {
                writeln!(self.output, "// TODO: Implement opcode {:?}", instruction.opcode).unwrap();
            }
        }

        Ok(())
    }

    fn generate_terminator(&mut self, terminator: &Terminator) -> Result<(), CompileError> {
        self.write_indent();

        match terminator {
            Terminator::Return(value) => {
                if let Some(value_id) = value {
                    let var_name = self.get_value_name(*value_id);
                    writeln!(self.output, "return {};", var_name).unwrap();
                } else {
                    writeln!(self.output, "return;").unwrap();
                }
            }
            Terminator::Jump(block_id) => {
                let label = &self.block_labels[block_id];
                writeln!(self.output, "goto {};", label).unwrap();
            }
            Terminator::Branch { condition, then_block, else_block } => {
                let condition_name = self.get_value_name(*condition);
                let then_label = &self.block_labels[then_block];
                let else_label = &self.block_labels[else_block];

                writeln!(self.output, "if ({}) {{", condition_name).unwrap();
                self.indent_level += 1;
                self.write_indent();
                writeln!(self.output, "goto {};", then_label).unwrap();
                self.indent_level -= 1;
                self.write_indent();
                writeln!(self.output, "}} else {{").unwrap();
                self.indent_level += 1;
                self.write_indent();
                writeln!(self.output, "goto {};", else_label).unwrap();
                self.indent_level -= 1;
                self.write_indent();
                writeln!(self.output, "}}").unwrap();
            }
            Terminator::Unreachable => {
                writeln!(self.output, "// Unreachable code").unwrap();
                writeln!(self.output, "abort();").unwrap();
            }
        }

        Ok(())
    }

    // Helper methods

    fn tir_type_to_c(&self, tir_type: &TirType) -> String {
        match tir_type {
            TirType::Int(8) => "int8_t".to_string(),
            TirType::Int(16) => "int16_t".to_string(),
            TirType::Int(32) => "int32_t".to_string(),
            TirType::Int(64) => "int64_t".to_string(),
            TirType::Int(_) => "int".to_string(),
            TirType::Float(32) => "float".to_string(),
            TirType::Float(64) => "double".to_string(),
            TirType::Float(_) => "double".to_string(),
            TirType::Bool => "bool".to_string(),
            TirType::Unit => "void".to_string(),
            TirType::Pointer(inner) => format!("{}*", self.tir_type_to_c(inner)),
            TirType::Array(inner, size) => format!("{}[{}]", self.tir_type_to_c(inner), size),
            TirType::Struct(name) => name.clone(),
            TirType::Enum(name) => name.clone(),
            TirType::Function(_) => "void*".to_string(), // Function pointer
        }
    }

    fn constant_to_c(&self, value: &ConstantValue) -> String {
        match value {
            ConstantValue::Int(i) => i.to_string(),
            ConstantValue::Float(f) => f.to_string(),
            ConstantValue::Bool(b) => b.to_string(),
            ConstantValue::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
            ConstantValue::Unit => "0".to_string(),
        }
    }

    fn get_value_name(&mut self, value_id: ValueId) -> String {
        if let Some(name) = self.value_names.get(&value_id) {
            name.clone()
        } else {
            let name = format!("v{}", self.value_counter);
            self.value_counter += 1;
            self.value_names.insert(value_id, name.clone());
            name
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            write!(self.output, "    ").unwrap();
        }
    }
}

// Plugin registration function
#[no_mangle]
pub extern "C" fn register_backend() -> Box<dyn Backend> {
    Box::new(CBackend::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::tir::{TirModule, TirFunction, FunctionSignature, CallingConvention, BasicBlock, BlockId, FunctionId};

    #[test]
    fn test_simple_function_generation() {
        let mut module = TirModule::new("test".to_string());

        let signature = FunctionSignature {
            parameters: vec![],
            return_type: TirType::Unit,
            calling_convention: CallingConvention::Default,
        };

        let mut function = TirFunction::new(FunctionId(0), "test_func".to_string(), signature);
        let entry_block = BasicBlock::new(BlockId(0));
        function.add_block(entry_block);

        module.add_function(function);

        let backend = CBackend::default();
        let result = backend.compile(module).unwrap();

        if let Ok(artifact) = result.downcast::<CompiledArtifact>() {
            let c_code = String::from_utf8(artifact.data).unwrap();
            assert!(c_code.contains("void test_func(void)"));
        } else {
            panic!("Expected CompiledArtifact");
        }
    }
}