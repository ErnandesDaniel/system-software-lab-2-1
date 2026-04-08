use crate::ir::*;

pub struct CfgMermaidGenerator {
    output: String,
}

impl CfgMermaidGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    pub fn generate(&mut self, program: &IrProgram) -> String {
        self.output.clear();
        self.output.push_str("flowchart TD\n");

        for func in &program.functions {
            self.generate_function(func);
        }

        std::mem::take(&mut self.output)
    }

    fn generate_function(&mut self, func: &IrFunction) {
        self.output
            .push_str(&format!("\n subgraph {}\n", func.name));

        for block in &func.blocks {
            self.generate_block(block);
        }

        self.output.push_str(" end\n");

        for block in &func.blocks {
            for succ in &block.successors {
                self.output
                    .push_str(&format!("{} --> {}\n", block.id, succ));
            }
        }
    }

    fn generate_block(&mut self, block: &IrBlock) {
        self.output
            .push_str(&format!("    {}[\"{}\"]\n", block.id, block.id));

        if block.instructions.is_empty() {
            return;
        }

        self.output
            .push_str(&format!("    {} --> {}_inst\n", block.id, block.id));
        self.output.push_str(&format!(
            "    {}_inst{{\"\\n{} \\n\"}}\n",
            block.id,
            self.format_block_instructions(block)
        ));
    }

    fn format_block_instructions(&self, block: &IrBlock) -> String {
        block
            .instructions
            .iter()
            .map(|inst| self.format_instruction(inst))
            .collect::<Vec<_>>()
            .join("\\n")
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
                format!("{} = {} == {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Ne => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} != {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Lt => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} < {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Gt => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} > {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Le => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} <= {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
            IrOpcode::Ge => {
                let (a, b) = self.get_operands(inst);
                format!("{} = {} >= {}", inst.result.as_deref().unwrap_or("?"), a, b)
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
                let a = self.get_single_operand(inst);
                format!("{} = {}", inst.result.as_deref().unwrap_or("?"), a)
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
                format!("jump {}", target)
            }
            IrOpcode::CondBr => {
                let cond = self.get_single_operand(inst);
                format!(
                    "br {} ? {} : {}",
                    cond,
                    inst.true_target.as_deref().unwrap_or("?"),
                    inst.false_target.as_deref().unwrap_or("?")
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
                format!("{} = {} | {}", inst.result.as_deref().unwrap_or("?"), a, b)
            }
        }
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
                Constant::String(s) => format!("\"{}\"", s),
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
