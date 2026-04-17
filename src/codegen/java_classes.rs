use crate::ir::*;
use std::collections::HashMap;

pub struct JavaClassGenerator {
    classes: HashMap<String, JavaClass>,
    current_class: Option<String>,
}

#[derive(Clone)]
pub struct JavaClass {
    pub name: String,
    pub super_class: String,
    pub interfaces: Vec<String>,
    pub fields: Vec<JavaFieldDef>,
    pub methods: Vec<JavaMethodDef>,
}

#[derive(Clone)]
pub struct JavaFieldDef {
    pub name: String,
    pub descriptor: String,
    pub access_flags: u16,
}

#[derive(Clone)]
pub struct JavaMethodDef {
    pub name: String,
    pub descriptor: String,
    pub access_flags: u16,
    pub instructions: Vec<JavaInstruction>,
}

#[derive(Clone)]
pub struct JavaInstruction {
    pub opcode: u8,
    pub operands: Vec<u8>,
}

impl JavaClassGenerator {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            current_class: None,
        }
    }

    pub fn generate(&mut self, program: &IrProgram) -> Vec<u8> {
        let class = self.create_class_from_ir(program);
        self.write_class(&class)
    }

    fn create_class_from_ir(&mut self, program: &IrProgram) -> JavaClass {
        let mut class = JavaClass {
            name: "MyLang".to_string(),
            super_class: "java/lang/Object".to_string(),
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
        };

        for func in &program.functions {
            let method = self.ir_function_to_java_method(func);
            class.methods.push(method);
        }

        class
    }

    fn ir_function_to_java_method(&self, func: &IrFunction) -> JavaMethodDef {
        let mut instructions = Vec::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                let java_inst = self.ir_instruction_to_java(inst);
                instructions.push(java_inst);
            }
        }

        let descriptor = self.build_method_descriptor(func);

        JavaMethodDef {
            name: func.name.clone(),
            descriptor,
            access_flags: if func.name == "main" { 0x09 } else { 0x01 },
            instructions,
        }
    }

    fn ir_instruction_to_java(&self, inst: &IrInstruction) -> JavaInstruction {
        let opcode = match inst.opcode {
            IrOpcode::Add => 0x60,
            IrOpcode::Sub => 0x64,
            IrOpcode::Mul => 0x68,
            IrOpcode::Div => 0x6C,
            IrOpcode::Mod => 0x70,
            IrOpcode::Eq => 0x9F,
            IrOpcode::Ne => 0xA0,
            IrOpcode::Lt => 0xA1,
            IrOpcode::Le => 0xA2,
            IrOpcode::Gt => 0xA3,
            IrOpcode::Ge => 0xA4,
            IrOpcode::And => 0x7E,
            IrOpcode::Or => 0x80,
            IrOpcode::Not => 0x95,
            IrOpcode::Neg => 0x74,
            IrOpcode::BitNot => 0x92,
            IrOpcode::BitAnd => 0x7E,
            IrOpcode::BitOr => 0x80,
            IrOpcode::Jump => 0xA7,
            IrOpcode::CondBr => 0x99,
            IrOpcode::Ret => 0xAC,
            IrOpcode::Load => 0x1C,
            IrOpcode::Store => 0x36,
            IrOpcode::Assign => 0x3C,
            _ => 0x00,
        };

        JavaInstruction {
            opcode,
            operands: Vec::new(),
        }
    }

    fn build_method_descriptor(&self, func: &IrFunction) -> String {
        let mut desc = String::from("(");
        for param in &func.parameters {
            desc.push_str(&self.ir_type_to_jvm(&param.ty));
        }
        desc.push(')');
        desc.push_str(&self.ir_type_to_jvm(&func.return_type));
        desc
    }

    fn ir_type_to_jvm(&self, ty: &IrType) -> String {
        match ty {
            IrType::Void => "V".to_string(),
            IrType::Bool | IrType::Int => "I".to_string(),
            IrType::String => "Ljava/lang/String;".to_string(),
            IrType::Array(_, _) => "[I".to_string(),
        }
    }

    fn write_class(&self, class: &JavaClass) -> Vec<u8> {
        let mut data = Vec::new();

        data.extend_from_slice(&[0xCA, 0xFE, 0xBA, 0xBE]);
        data.extend_from_slice(&0x00_00_00_34u32.to_be_bytes());

        let cp_count = 20u16;
        data.extend_from_slice(&cp_count.to_be_bytes());

        data.extend_from_slice(&[0x0A, 0x00, 0x03, 0x00, 0x0A]);
        data.extend_from_slice(&[0x07, 0x00, 0x0B]);
        data.extend_from_slice(&[0x07, 0x00, 0x0C]);

        data.extend_from_slice(&[0x01, 0x00, 0x10, 0x6A, 0x61, 0x76, 0x61, 0x2F, 0x6C, 0x61, 0x6E, 0x67, 0x2F, 0x4F, 0x62, 0x6A, 0x65, 0x63, 0x74]);
        data.extend_from_slice(&[0x01, 0x00, 0x01, 0x3C, 0x69, 0x6E, 0x69, 0x74, 0x3E]);
        data.extend_from_slice(&[0x01, 0x00, 0x03, 0x28, 0x29, 0x56]);
        data.extend_from_slice(&[0x0C, 0x00, 0x07, 0x00, 0x08]);
        data.extend_from_slice(&[0x0A, 0x00, 0x03, 0x00, 0x09]);
        data.extend_from_slice(&[0x07, 0x00, 0x0D]);
        data.extend_from_slice(&[0x01, 0x00, 0x06, 0x3C, 0x63, 0x6C, 0x69, 0x6E, 0x69, 0x74, 0x3E]);
        data.extend_from_slice(&[0x01, 0x00, 0x05, 0x4D, 0x79, 0x4C, 0x61, 0x6E, 0x67]);
        data.extend_from_slice(&[0x01, 0x00, 0x04, 0x6D, 0x61, 0x69, 0x6E]);
        data.extend_from_slice(&[0x01, 0x00, 0x03, 0x28, 0x29, 0x56]);
        data.extend_from_slice(&[0x01, 0x00, 0x04, 0x43, 0x6F, 0x64, 0x65]);
        data.extend_from_slice(&[0x01, 0x00, 0x0D, 0x4C, 0x6A, 0x61, 0x76, 0x61, 0x2F, 0x69, 0x6F, 0x2F, 0x50, 0x72, 0x69, 0x6E, 0x74, 0x53, 0x74, 0x72, 0x65, 0x61, 0x6D, 0x3B]);
        data.extend_from_slice(&[0x0A, 0x00, 0x0E, 0x00, 0x0F]);

        data.extend_from_slice(&0x0021u16.to_be_bytes());
        data.extend_from_slice(&0x0000u16.to_be_bytes());

        let method_count = class.methods.len() as u16;
        data.extend_from_slice(&method_count.to_be_bytes());

        for method in &class.methods {
            data.extend_from_slice(&method.access_flags.to_be_bytes());
            data.extend_from_slice(&self.add_16(class.name.len() as u16).to_be_bytes());
            data.extend_from_slice(&self.add_16(3).to_be_bytes());

            let attr_count = 1u16;
            data.extend_from_slice(&attr_count.to_be_bytes());

            data.extend_from_slice(&0x0000000Du32.to_be_bytes());
            data.extend_from_slice(&0x0001u16.to_be_bytes());
            data.extend_from_slice(&0x0001u16.to_be_bytes());

            let code_len = method.instructions.len() as u32 + 12;
            data.extend_from_slice(&code_len.to_be_bytes());

            data.extend_from_slice(&0x00u8.to_be_bytes());
            for _ in &method.instructions {
                data.extend_from_slice(&0x00u8.to_be_bytes());
            }

            data.extend_from_slice(&0x0000u16.to_be_bytes());
            data.extend_from_slice(&0x0000u16.to_be_bytes());
        }

        let attr_count = 0u16;
        data.extend_from_slice(&attr_count.to_be_bytes());

        data
    }

    fn add_16(&self, value: u16) -> u16 {
        value
    }
}

impl Default for JavaClassGenerator {
    fn default() -> Self {
        Self::new()
    }
}