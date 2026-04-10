use crate::ir::*;

pub struct CfgMermaidGenerator {
    output: String,
}

#[allow(dead_code)]
impl CfgMermaidGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn generate(&mut self, program: &IrProgram) -> String {
        self.output.clear();
        self.output.push_str("graph TD\n");
        for func in &program.functions {
            self.generate_function(func);
        }
        std::mem::take(&mut self.output)
    }

    pub fn generate_function_only(&mut self, func: &IrFunction) -> String {
        self.output.clear();
        self.output.push_str("graph TD\n");
        self.generate_function(func);
        std::mem::take(&mut self.output)
    }

    fn generate_function(&mut self, func: &IrFunction) {
        for block in &func.blocks {
            self.generate_block(block);
        }

        for block in &func.blocks {
            for succ in &block.successors {
                self.output.push_str(&format!(
                    "{} --> {}\n",
                    self.format_block_id(&block.id),
                    self.format_block_id(succ)
                ));
            }
        }
    }

    fn format_block_id(&self, id: &str) -> String {
        id.replace("BB", "BB_")
    }

    fn generate_block(&mut self, block: &IrBlock) {
        let formatted_id = self.format_block_id(&block.id);
        let instructions = self.format_block_instructions(block);

        self.output.push_str(&format!(
            "    {}[\"{}\\n{}\"]\n",
            formatted_id, block.id, instructions
        ));
    }

    fn format_block_instructions(&self, block: &IrBlock) -> String {
        let raw = block
            .instructions
            .iter()
            .map(|inst| self.format_instruction(inst))
            .collect::<Vec<_>>()
            .join("\n");

        // Escape special characters for Mermaid diagram
        // Note: don't escape parentheses () as they work fine in mermaid node text
        raw.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('{', "\\{")
            .replace('}', "\\}")
    }

    fn format_instruction(&self, inst: &IrInstruction) -> String {
        match &inst.opcode {
            IrOpcode::Add => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} + {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Sub => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} - {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Mul => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} * {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Div => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} / {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Mod => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} % {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Eq => {
                let (a, b) = self.get_operands(inst);
                format!(
                    "{} = ({})",
                    inst.result.as_deref().unwrap_or("?"),
                    self.format_cmp(&a, &b, "==")
                )
            }
            IrOpcode::Ne => {
                let (a, b) = self.get_operands(inst);
                format!(
                    "{} = ({})",
                    inst.result.as_deref().unwrap_or("?"),
                    self.format_cmp(&a, &b, "!=")
                )
            }
            IrOpcode::Lt => {
                let (a, b) = self.get_operands(inst);
                format!(
                    "{} = ({})",
                    inst.result.as_deref().unwrap_or("?"),
                    self.format_cmp(&a, &b, "<")
                )
            }
            IrOpcode::Gt => {
                let (a, b) = self.get_operands(inst);
                format!(
                    "{} = ({})",
                    inst.result.as_deref().unwrap_or("?"),
                    self.format_cmp(&a, &b, ">")
                )
            }
            IrOpcode::Le => {
                let (a, b) = self.get_operands(inst);
                format!(
                    "{} = ({})",
                    inst.result.as_deref().unwrap_or("?"),
                    self.format_cmp(&a, &b, "<=")
                )
            }
            IrOpcode::Ge => {
                let (a, b) = self.get_operands(inst);
                format!(
                    "{} = ({})",
                    inst.result.as_deref().unwrap_or("?"),
                    self.format_cmp(&a, &b, ">=")
                )
            }
            IrOpcode::And => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} && {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Or => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} || {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Not => {
                let a = self.get_single_operand(inst);
                format!("{} = !{}", inst.result.as_deref().unwrap_or("?"), a)
            }
            IrOpcode::Neg => {
                let a = self.get_single_operand(inst);
                format!("{} = -{}", inst.result.as_deref().unwrap_or("?"), a)
            }
            IrOpcode::Pos => {
                let a = self.get_single_operand(inst);
                format!("{} = +{}", inst.result.as_deref().unwrap_or("?"), a)
            }
            IrOpcode::BitNot => {
                let a = self.get_single_operand(inst);
                format!("{} = ~{}", inst.result.as_deref().unwrap_or("?"), a)
            }
            IrOpcode::Assign => {
                if let Some(ref result) = inst.result {
                    if let Some(op) = inst.operands.first() {
                        let a = self.format_operand(op);
                        if a == *result {
                            return format!("{}", a);
                        }
                        return format!("{} = {}", result, a);
                    }
                }
                "=".to_string()
            }
            IrOpcode::Call => {
                let target = inst.jump_target.as_deref().unwrap_or("?");
                let args = inst
                    .operands
                    .iter()
                    .map(|op| self.format_operand(op))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!(
                    "{} = {}({})",
                    inst.result.as_deref().unwrap_or("?"),
                    target,
                    args
                )
            }
            IrOpcode::Jump => {
                let target = inst.jump_target.as_deref().unwrap_or("?");
                format!("goto {}", self.format_block_id(target))
            }
            IrOpcode::CondBr => {
                let cond = self.get_single_operand(inst);
                format!(
                    "if {} goto {} else {}",
                    cond,
                    self.format_block_id(inst.true_target.as_deref().unwrap_or("?")),
                    self.format_block_id(inst.false_target.as_deref().unwrap_or("?"))
                )
            }
            IrOpcode::Ret => {
                if inst.operands.is_empty() {
                    "return".to_string()
                } else {
                    let a = self.get_single_operand(inst);
                    format!("return {}", a)
                }
            }
            IrOpcode::Load => {
                let (arr, idx) = self.get_operands(inst);
                format!(
                    "{} = {}[{}]",
                    inst.result.as_deref().unwrap_or("?"),
                    arr,
                    idx
                )
            }
            IrOpcode::Store => {
                let (arr, idx) = self.get_operands(inst);
                format!(
                    "{}[{}] = {}",
                    arr,
                    idx,
                    inst.result.as_deref().unwrap_or("?")
                )
            }
            IrOpcode::Slice => {
                let (arr, start) = self.get_operands(inst);
                format!(
                    "{} = {}[{}:]",
                    inst.result.as_deref().unwrap_or("?"),
                    arr,
                    start
                )
            }
            IrOpcode::Alloca => {
                format!("alloca {}", inst.result.as_deref().unwrap_or("?"))
            }
            IrOpcode::Cast => {
                let a = self.get_single_operand(inst);
                format!("{} = cast({})", inst.result.as_deref().unwrap_or("?"), a)
            }
            IrOpcode::BitAnd | IrOpcode::BitOr => {
                let (a, b) = self.get_operands(inst);
                let op = if matches!(inst.opcode, IrOpcode::BitAnd) {
                    "&"
                } else {
                    "|"
                };
                format!(
                    "{} = {} {} {}",
                    inst.result.as_deref().unwrap_or("?"),
                    a,
                    op,
                    b
                )
            }
            IrOpcode::CreateThread => {
                format!(
                    "create_thread({:?}, {:?})",
                    inst.operands.get(0).map(|o| format!("{:?}", o)),
                    inst.operands.get(1).map(|o| format!("{:?}", o))
                )
            }
            IrOpcode::Yield => "yield".to_string(),
            IrOpcode::CoroutineCreate => "coroutine_create".to_string(),
            IrOpcode::CoroutineYield => "coroutine_yield".to_string(),
            IrOpcode::CoroutineResume => "coroutine_resume".to_string(),
        }
    }

    fn format_cmp(&self, a: &str, b: &str, op: &str) -> String {
        format!("{} {} {}", a, op, b)
    }

    fn get_operands(&self, inst: &IrInstruction) -> (String, String) {
        let a = inst
            .operands
            .first()
            .map(|op| self.format_operand(op))
            .unwrap_or_else(|| "?".to_string());
        let b = inst
            .operands
            .get(1)
            .map(|op| self.format_operand(op))
            .unwrap_or_else(|| "?".to_string());
        (a, b)
    }

    fn get_single_operand(&self, inst: &IrInstruction) -> String {
        inst.operands
            .first()
            .map(|op| self.format_operand(op))
            .unwrap_or_else(|| "?".to_string())
    }

    fn format_operand(&self, op: &IrOperand) -> String {
        match op {
            IrOperand::Variable(name, _) => name.clone(),
            IrOperand::Constant(c) => match c {
                Constant::Int(n) => n.to_string(),
                Constant::Bool(b) => b.to_string(),
                Constant::String(s) => s
                    .replace('\\', "\\\\")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t"),
                Constant::Char(c) => format!("'{}'", *c as char),
            },
        }
    }
}

impl Default for CfgMermaidGenerator {
    fn default() -> Self {
        Self::new()
    }
}
