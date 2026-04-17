use crate::ir::*;
use std::collections::HashMap;

pub struct JavaSourceGenerator {
    functions: Vec<FunctionInfo>,
    locals: HashMap<String, usize>,
}

#[derive(Clone)]
struct FunctionInfo {
    name: String,
    params: Vec<(String, IrType)>,
    return_type: IrType,
    statements: Vec<String>,
}

impl JavaSourceGenerator {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            locals: HashMap::new(),
        }
    }

    pub fn generate(&mut self, program: &IrProgram) -> String {
        self.functions.clear();
        self.locals.clear();
        
        for func in &program.functions {
            self.generate_function(func);
        }

        self.build_java_source()
    }

    fn generate_function(&mut self, func: &IrFunction) {
        let mut statements = Vec::new();
        self.locals.clear();

        for local in &func.locals {
            if local.name.starts_with('t') {
                let idx = self.locals.len();
                self.locals.insert(local.name.clone(), idx);
            }
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                let stmt = self.translate_instruction(inst, func);
                if !stmt.is_empty() {
                    statements.push(stmt);
                }
            }
        }

        let params: Vec<(String, IrType)> = func.parameters
            .iter()
            .map(|p| (p.name.clone(), p.ty.clone()))
            .collect();

        let func_info = FunctionInfo {
            name: func.name.clone(),
            params,
            return_type: func.return_type.clone(),
            statements,
        };

        self.functions.push(func_info);
    }

    fn translate_instruction(&self, inst: &IrInstruction, func: &IrFunction) -> String {
        match inst.opcode {
            IrOpcode::Assign => {
                if let (Some(result), Some(operand)) = (&inst.result, inst.operands.first()) {
                    let rhs = self.translate_operand(operand, func);
                    return format!("int {} = {};", result, rhs);
                }
                String::new()
            }
            IrOpcode::Add => {
                if let (Some(result), Some(left), Some(right)) =
                    (&inst.result, inst.operands.get(0), inst.operands.get(1))
                {
                    let l = self.translate_operand(left, func);
                    let r = self.translate_operand(right, func);
                    return format!("int {} = {} + {};", result, l, r);
                }
                String::new()
            }
            IrOpcode::Sub => {
                if let (Some(result), Some(left), Some(right)) =
                    (&inst.result, inst.operands.get(0), inst.operands.get(1))
                {
                    let l = self.translate_operand(left, func);
                    let r = self.translate_operand(right, func);
                    return format!("int {} = {} - {};", result, l, r);
                }
                String::new()
            }
            IrOpcode::Mul => {
                if let (Some(result), Some(left), Some(right)) =
                    (&inst.result, inst.operands.get(0), inst.operands.get(1))
                {
                    let l = self.translate_operand(left, func);
                    let r = self.translate_operand(right, func);
                    return format!("int {} = {} * {};", result, l, r);
                }
                String::new()
            }
            IrOpcode::Div => {
                if let (Some(result), Some(left), Some(right)) =
                    (&inst.result, inst.operands.get(0), inst.operands.get(1))
                {
                    let l = self.translate_operand(left, func);
                    let r = self.translate_operand(right, func);
                    return format!("int {} = {} / {};", result, l, r);
                }
                String::new()
            }
            IrOpcode::Mod => {
                if let (Some(result), Some(left), Some(right)) =
                    (&inst.result, inst.operands.get(0), inst.operands.get(1))
                {
                    let l = self.translate_operand(left, func);
                    let r = self.translate_operand(right, func);
                    return format!("int {} = {} % {};", result, l, r);
                }
                String::new()
            }
            IrOpcode::Neg => {
                if let Some(ref result) = inst.result {
                    if let Some(operand) = inst.operands.first() {
                        let v = self.translate_operand(operand, func);
                        return format!("int {} = -{};", result, v);
                    }
                }
                String::new()
            }
            IrOpcode::Call => {
                if let Some(ref func_name) = inst.jump_target {
                    let args: Vec<String> = inst.operands
                        .iter()
                        .map(|a| self.translate_operand(a, func))
                        .collect();
                    let args_str = args.join(", ");
                    return format!("MyLangRuntime.{}({});", func_name, args_str);
                }
                String::new()
            }
            IrOpcode::Ret => {
                if let Some(operand) = inst.operands.first() {
                    let v = self.translate_operand(operand, func);
                    if func.return_type != IrType::Void {
                        return format!("return {};", v);
                    }
                }
                "return;".to_string()
            }
            _ => String::new()
        }
    }

    fn translate_operand(&self, operand: &IrOperand, _func: &IrFunction) -> String {
        match operand {
            IrOperand::Variable(name, _) => name.clone(),
            IrOperand::Constant(c) => match c {
                Constant::Int(v) => v.to_string(),
                Constant::Bool(b) => b.to_string(),
                Constant::String(s) => format!("\"{}\"", s),
                Constant::Char(c) => (*c as i64).to_string(),
            },
        }
    }

    fn build_java_source(&self) -> String {
        let mut source = String::new();

        source.push_str("public class MyLang {\n");
        source.push_str("    public static void main(String[] args) {\n");
        
        for func in &self.functions {
            if func.name == "main" {
                for stmt in &func.statements {
                    source.push_str(&format!("        {}\n", stmt));
                }
            }
        }
        
        source.push_str("    }\n\n");
        
        for func in &self.functions {
            if func.name != "main" {
                let params: Vec<String> = func.params
                    .iter()
                    .map(|(n, t)| format!("int {}", n))
                    .collect();
                
                source.push_str(&format!(
                    "    public static int {}({}) {{\n",
                    func.name,
                    params.join(", ")
                ));
                
                for stmt in &func.statements {
                    source.push_str(&format!("        {}\n", stmt));
                }
                
                source.push_str("    }\n\n");
            }
        }
        
        source.push_str("}\n");
        
        source
    }
}

impl Default for JavaSourceGenerator {
    fn default() -> Self {
        Self::new()
    }
}