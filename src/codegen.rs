use crate::ir::*;
use std::collections::HashMap;

pub struct AsmGenerator {
    output: String,
    string_counter: usize,
    data_section: String,
    locals: HashMap<String, i32>,
    temps: HashMap<String, i32>,
    used_functions: Vec<String>,
    current_function: Option<String>,
    param_registers: Vec<String>,
    temp_counter: usize,
}

impl AsmGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            string_counter: 0,
            data_section: String::new(),
            locals: HashMap::new(),
            temps: HashMap::new(),
            used_functions: Vec::new(),
            current_function: None,
            param_registers: vec![
                "rcx".to_string(),
                "rdx".to_string(),
                "r8".to_string(),
                "r9".to_string(),
            ],
            temp_counter: 0,
        }
    }

    pub fn generate(&mut self, program: &IrProgram) -> String {
        self.output.push_str("bits 64\n");
        self.output.push_str("default rel\n");
        self.output.push_str("section .text\n\n");

        // Generate extern declarations for all used external functions
        let mut extern_funcs: std::collections::HashSet<&String> = std::collections::HashSet::new();
        for func in &program.functions {
            for ext_func in &func.used_functions {
                extern_funcs.insert(ext_func);
            }
        }
        for ext_func in extern_funcs {
            self.output.push_str(&format!("extern {}\n", ext_func));
        }
        if !program.functions.is_empty() {
            self.output.push_str("\n");
        }

        for func in &program.functions {
            self.generate_function_internal(func);
        }

        if !self.data_section.is_empty() {
            self.output.push_str("\nsection .data\n");
            self.output.push_str(&self.data_section);
        }

        self.output.clone()
    }

    pub fn generate_single_function(&mut self, func: &IrFunction) -> String {
        self.output.clear();
        self.string_counter = 0;
        self.data_section.clear();

        // Collect unique extern functions for this function
        let mut unique_externs: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for ext_func in &func.used_functions {
            unique_externs.insert(ext_func.clone());
        }

        self.output.push_str("bits 64\n");
        self.output.push_str("default rel\n");
        self.output.push_str("section .text\n\n");

        // Generate global and extern declarations
        self.output.push_str(&format!("global {}\n", func.name));

        // Collect unique externs in sorted order for consistency
        let mut externs: Vec<_> = unique_externs.into_iter().collect();
        externs.sort();
        for ext_func in &externs {
            self.output.push_str(&format!("extern {}\n", ext_func));
        }

        if !externs.is_empty() {
            self.output.push_str("\n");
        }

        // Setup locals and temps tracking
        self.current_function = Some(func.name.clone());
        self.locals.clear();
        self.temps.clear();
        self.temp_counter = 0;

        // Assign offsets to LOCAL variables (starting from -8)
        let mut local_counter: i32 = 1;
        for local in &func.locals {
            let offset = -8 * local_counter;
            local_counter += 1;
            self.locals.insert(local.name.clone(), offset);
        }

        // Track temp variables for code generation
        // They don't use stack space, so use negative offsets starting after locals
        let mut temp_offset: i32 = -(8 * local_counter);
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref result) = inst.result {
                    if result.starts_with('t') && !self.temps.contains_key(result) {
                        self.temps.insert(result.clone(), temp_offset);
                        temp_offset -= 8;
                    }
                }
            }
        }

        // Calculate stack size: from -8 to the most negative temp
        let total_vars = local_counter + ((-temp_offset / 8) - local_counter);
        let stack_size = -8 * total_vars;
        let abs_stack = if stack_size < 0 {
            -stack_size
        } else {
            stack_size
        };
        let aligned = ((abs_stack + 15) / 16) * 16;
        let final_stack = aligned.max(16);

        self.output.push_str(&format!("{}:\n", func.name));
        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");
        self.output
            .push_str(&format!("    sub rsp, {}\n", final_stack));

        // Generate all blocks
        for block in &func.blocks {
            self.generate_block(block);
        }

        if !self.data_section.is_empty() {
            self.output.push_str("\nsection .data\n");
            self.output.push_str(&self.data_section);
        }

        std::mem::take(&mut self.output)
    }

    fn generate_function_internal(&mut self, func: &IrFunction) {
        self.current_function = Some(func.name.clone());
        self.locals.clear();
        self.temps.clear();
        self.temp_counter = 0;

        // First pass: collect all locals and count temps
        // Regular locals go to locals map, temps go to temps map
        for local in &func.locals {
            let offset = local.stack_offset.unwrap_or_else(|| {
                // Assign offset for locals without one
                let off = -8 * (self.locals.len() as i32 + self.temps.len() as i32 + 1);
                off
            });

            // Check if this is a temp variable (starts with 't')
            if local.name.starts_with('t') {
                self.temps.insert(local.name.clone(), offset);
            } else {
                self.locals.insert(local.name.clone(), offset);
            }
        }

        // Count temp variables (t0, t1, etc.) used in instructions
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref result) = inst.result {
                    if result.starts_with('t') {
                        if !self.temps.contains_key(result) {
                            let offset = -8 * (self.temp_counter as i32 + 1);
                            self.temps.insert(result.clone(), offset);
                            self.temp_counter += 1;
                        }
                    }
                }
            }
        }

        self.param_registers.clear();
        for (i, param) in func.parameters.iter().enumerate() {
            if i < 4 {
                let _ = match i {
                    0 => "rcx",
                    1 => "rdx",
                    2 => "r8",
                    3 => "r9",
                    _ => "",
                };
                self.param_registers.push(param.name.clone());
            }
        }

        self.output.push_str(&format!("global {}\n", func.name));
        self.output.push_str(&format!("{}:\n", func.name));
        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");

        let frame_size = self.calculate_frame_size(func);
        if frame_size > 0 {
            self.output
                .push_str(&format!("    sub rsp, {}\n", frame_size));
        }

        for block in &func.blocks {
            self.generate_block(block);
        }

        self.output.push_str("    leave\n");
        self.output.push_str("    ret\n");
    }

    fn calculate_frame_size(&self, func: &IrFunction) -> i32 {
        let mut size = 0;
        for local in &func.locals {
            if let Some(offset) = local.stack_offset {
                size = size.max(-offset);
            }
        }

        let aligned = ((size + 15) / 16) * 16;
        if aligned == 0 && size > 0 {
            aligned + 8
        } else {
            aligned
        }
    }

    fn generate_block(&mut self, block: &IrBlock) {
        let label = if block.id.starts_with("BB") {
            format!("BB_{}", block.id.trim_start_matches("BB"))
        } else {
            block.id.clone()
        };
        self.output.push_str(&format!("{}:\n", label));

        for inst in &block.instructions {
            self.generate_instruction(inst);
        }
    }

    fn generate_instruction(&mut self, inst: &IrInstruction) {
        match inst.opcode {
            IrOpcode::Assign => {
                if let Some(ref result) = inst.result {
                    if let Some(operand) = inst.operands.first() {
                        // Check if operand type is pointer (string)
                        let is_pointer = operand.get_type().is_pointer();
                        // Always use rax for pointers to preserve full 64-bit address
                        if is_pointer {
                            self.load_operand(operand, "rax", true);
                            self.store_variable(result, "rax", true);
                        } else {
                            self.load_operand(operand, "eax", false);
                            self.store_variable(result, "eax", false);
                        }
                    }
                }
            }
            IrOpcode::Add => self.binary_op(inst, "add"),
            IrOpcode::Sub => self.binary_op(inst, "sub"),
            IrOpcode::Mul => self.binary_op(inst, "imul"),
            IrOpcode::Div => self.binary_op_div(inst),
            IrOpcode::Mod => self.binary_op_mod(inst),
            IrOpcode::Eq => self.compare_op(inst, "sete"),
            IrOpcode::Ne => self.compare_op(inst, "setne"),
            IrOpcode::Lt => self.compare_op(inst, "setl"),
            IrOpcode::Le => self.compare_op(inst, "setle"),
            IrOpcode::Gt => self.compare_op(inst, "setg"),
            IrOpcode::Ge => self.compare_op(inst, "setge"),
            IrOpcode::And => self.binary_op(inst, "and"),
            IrOpcode::Or => self.binary_op(inst, "or"),
            IrOpcode::Not => {
                if let Some(ref result) = inst.result {
                    if let Some(operand) = inst.operands.first() {
                        self.load_operand(operand, "eax", false);
                        self.output.push_str("    test eax, eax\n");
                        self.output.push_str("    setz al\n");
                        self.output.push_str("    movzx eax, al\n");
                        self.store_variable(result, "eax", false);
                    }
                }
            }
            IrOpcode::BitNot => {
                if let Some(ref result) = inst.result {
                    if let Some(operand) = inst.operands.first() {
                        self.load_operand(operand, "eax", false);
                        self.output.push_str("    not eax\n");
                        self.store_variable(result, "eax", false);
                    }
                }
            }
            IrOpcode::Neg => {
                if let Some(ref result) = inst.result {
                    if let Some(operand) = inst.operands.first() {
                        self.load_operand(operand, "eax", false);
                        self.output.push_str("    neg eax\n");
                        self.store_variable(result, "eax", false);
                    }
                }
            }
            IrOpcode::Pos => {
                if let Some(ref result) = inst.result {
                    if let Some(operand) = inst.operands.first() {
                        self.load_operand(operand, "eax", false);
                        self.store_variable(result, "eax", false);
                    }
                }
            }
            IrOpcode::Call => self.generate_call(inst),
            IrOpcode::Jump => {
                if let Some(ref target) = inst.jump_target {
                    let formatted_target = self.format_block_label(target);
                    self.output
                        .push_str(&format!("    jmp {}\n", formatted_target));
                }
            }
            IrOpcode::CondBr => self.generate_cond_br(inst),
            IrOpcode::Ret => self.generate_ret(inst),
            IrOpcode::Load => self.generate_load(inst),
            IrOpcode::Store => self.generate_store(inst),
            IrOpcode::Slice => self.generate_slice(inst),
            IrOpcode::Alloca => {}
            IrOpcode::Cast => {
                if let Some(ref result) = inst.result {
                    if let Some(operand) = inst.operands.first() {
                        let is_pointer = operand.get_type().is_pointer();
                        self.load_operand(
                            operand,
                            if is_pointer { "rax" } else { "eax" },
                            is_pointer,
                        );
                        self.store_variable(
                            result,
                            if is_pointer { "rax" } else { "eax" },
                            is_pointer,
                        );
                    }
                }
            }
            _ => {}
        }
    }

    fn binary_op(&mut self, inst: &IrInstruction, op: &str) {
        if let (Some(result), Some(left), Some(right)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str(&format!("    {} eax, ebx\n", op));
            self.store_variable(result, "eax", false);
        }
    }

    fn binary_op_div(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(left, "eax", false);
            self.output.push_str("    cdq\n");
            self.load_operand(right, "ebx", false);
            self.output.push_str("    idiv ebx\n");
            self.store_variable(result, "eax", false);
        }
    }

    fn binary_op_mod(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(left, "eax", false);
            self.output.push_str("    cdq\n");
            self.load_operand(right, "ebx", false);
            self.output.push_str("    idiv ebx\n");
            self.output.push_str("    mov eax, edx\n");
            self.store_variable(result, "eax", false);
        }
    }

    fn compare_op(&mut self, inst: &IrInstruction, set_op: &str) {
        if let (Some(result), Some(left), Some(right)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cmp eax, ebx\n");
            self.output.push_str(&format!("    {} al\n", set_op));
            self.output.push_str("    movzx eax, al\n");
            self.store_variable(result, "eax", false);
        }
    }

    fn generate_call(&mut self, inst: &IrInstruction) {
        if let Some(ref func_name) = inst.jump_target {
            self.used_functions.push(func_name.clone());

            // For function calls, determine parameter registers
            // Pointers go in rcx, rdx, r8, r9
            // Integers go in ecx, edx, r8d, r9d

            // First, load all arguments into appropriate registers
            for (i, arg) in inst.operands.iter().enumerate() {
                if i < 4 {
                    let is_pointer = arg.get_type().is_pointer();

                    // Choose 64-bit register for pointer, 32-bit for integer
                    let load_reg = match i {
                        0 => {
                            if is_pointer {
                                "rcx"
                            } else {
                                "ecx"
                            }
                        }
                        1 => {
                            if is_pointer {
                                "rdx"
                            } else {
                                "edx"
                            }
                        }
                        2 => {
                            if is_pointer {
                                "r8"
                            } else {
                                "r8d"
                            }
                        }
                        3 => {
                            if is_pointer {
                                "r9"
                            } else {
                                "r9d"
                            }
                        }
                        _ => "ecx",
                    };

                    // Load operand into the register
                    // If pointer, we need 64-bit register for address
                    if is_pointer {
                        // Load directly into 64-bit register
                        match arg {
                            IrOperand::Constant(Constant::String(s)) => {
                                let label = format!("str_{}", self.string_counter);
                                self.string_counter += 1;
                                self.emit_string_data(&label, s);
                                self.output
                                    .push_str(&format!("    lea {}, [{}]\n", load_reg, label));
                            }
                            IrOperand::Variable(name, _) => {
                                // For string variables, load address using rax then mov to load_reg
                                if let Some(offset) = self.locals.get(name) {
                                    self.output
                                        .push_str(&format!("    mov rax, [rbp + {}]\n", offset));
                                    self.output
                                        .push_str(&format!("    mov {}, rax\n", load_reg));
                                } else if let Some(offset) = self.temps.get(name) {
                                    self.output
                                        .push_str(&format!("    mov rax, [rbp + {}]\n", offset));
                                    self.output
                                        .push_str(&format!("    mov {}, rax\n", load_reg));
                                }
                            }
                            _ => {
                                self.load_operand(arg, load_reg, true);
                            }
                        }
                    } else {
                        // Integer argument - use 32-bit register
                        self.load_operand(arg, load_reg, false);
                    }
                }
            }

            self.output.push_str("    sub rsp, 32\n");
            self.output.push_str(&format!("    call {}\n", func_name));
            self.output.push_str("    add rsp, 32\n");

            if let Some(ref result) = inst.result {
                let is_pointer = inst
                    .result_type
                    .as_ref()
                    .map(|t| t.is_pointer())
                    .unwrap_or(false);
                // For pointer results, use rax; for int results, use eax
                let reg = if is_pointer { "rax" } else { "eax" };
                self.store_variable(result, reg, is_pointer);
            }
        }
    }

    fn generate_cond_br(&mut self, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            self.load_operand(operand, "eax", false);
            self.output.push_str("    test eax, eax\n");

            if let (Some(ref true_t), Some(ref false_t)) = (&inst.true_target, &inst.false_target) {
                let formatted_true = self.format_block_label(true_t);
                let formatted_false = self.format_block_label(false_t);
                self.output
                    .push_str(&format!("    jne {}\n", formatted_true));
                self.output
                    .push_str(&format!("    jmp {}\n", formatted_false));
            }
        }
    }

    fn generate_ret(&mut self, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            match operand {
                IrOperand::Constant(c) => {
                    self.load_constant(c, "eax");
                }
                IrOperand::Variable(name, ty) => {
                    let is_pointer = ty.is_pointer();
                    // First check temps, then locals, then params
                    if let Some(offset) = self.temps.get(name) {
                        self.output
                            .push_str(&format!("    mov eax, [rbp + {}]\n", offset));
                    } else if let Some(offset) = self.locals.get(name) {
                        self.output
                            .push_str(&format!("    mov eax, [rbp + {}]\n", offset));
                    } else if self.param_registers.contains(name) {
                        let idx = self.param_registers.iter().position(|r| r == name).unwrap();
                        let src_reg = match idx {
                            0 => "ecx",
                            1 => "edx",
                            2 => "r8d",
                            3 => "r9d",
                            _ => "ecx",
                        };
                        self.output.push_str(&format!("    mov eax, {}\n", src_reg));
                    } else {
                        self.load_operand(operand, "eax", is_pointer);
                    }
                }
            }
        }
        self.output.push_str("; Очистка стека и возврат\n");
        self.output
            .push_str("    leave       ; эквивалент: mov rsp, rbp; pop rbp\n");
        self.output
            .push_str("    ret         ; возвращаем eax как результат\n");
    }

    fn generate_load(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(base), Some(index)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(base, "eax", true);
            self.load_operand(index, "ebx", false);
            self.output.push_str("    mov eax, [eax + ebx * 4]\n");
            self.store_variable(result, "eax", false);
        }
    }

    fn generate_store(&mut self, inst: &IrInstruction) {
        if let (Some(dest_name), Some(src), Some(_value)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            // Load source value into ebx
            self.load_operand(src, "ebx", false);
            // Store to destination - look up offset from locals/temps
            if let Some(offset) = self.locals.get(dest_name) {
                self.output
                    .push_str(&format!("    mov [rbp + {}], ebx\n", offset));
            } else if let Some(offset) = self.temps.get(dest_name) {
                self.output
                    .push_str(&format!("    mov [rbp + {}], ebx\n", offset));
            }
        }
    }

    fn generate_slice(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(base), Some(index)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            // Base is a pointer (array), use rax for it
            self.load_operand(base, "rax", true);
            self.load_operand(index, "ebx", false);
            self.output.push_str("    mov eax, [rax + ebx * 4]\n");
            self.store_variable(result, "eax", false);
        }
    }

    fn load_operand(&mut self, operand: &IrOperand, dest: &str, _is_pointer: bool) {
        match operand {
            IrOperand::Variable(name, ty) => {
                let is_ptr = ty.is_pointer();
                if let Some(offset) = self.locals.get(name) {
                    self.output.push_str(&format!(
                        "    mov {}, [rbp + {}]\n",
                        if is_ptr { "rax" } else { dest },
                        offset
                    ));
                } else if let Some(offset) = self.temps.get(name) {
                    self.output.push_str(&format!(
                        "    mov {}, [rbp + {}]\n",
                        if is_ptr { "rax" } else { dest },
                        offset
                    ));
                } else if self.param_registers.contains(name) {
                    let idx = self.param_registers.iter().position(|r| r == name).unwrap();
                    let src_reg = match idx {
                        0 => {
                            if is_ptr {
                                "rcx"
                            } else {
                                "ecx"
                            }
                        }
                        1 => {
                            if is_ptr {
                                "rdx"
                            } else {
                                "edx"
                            }
                        }
                        2 => {
                            if is_ptr {
                                "r8"
                            } else {
                                "r8d"
                            }
                        }
                        3 => {
                            if is_ptr {
                                "r9"
                            } else {
                                "r9d"
                            }
                        }
                        _ => {
                            if is_ptr {
                                "rcx"
                            } else {
                                "ecx"
                            }
                        }
                    };
                    self.output
                        .push_str(&format!("    mov {}, {}\n", dest, src_reg));
                } else {
                    self.output.push_str(&format!("    mov {}, 0\n", dest));
                }
            }
            IrOperand::Constant(c) => {
                self.load_constant(c, dest);
            }
        }
    }

    fn load_constant(&mut self, constant: &Constant, dest: &str) {
        match constant {
            Constant::Int(v) => {
                self.output.push_str(&format!("    mov {}, {}\n", dest, v));
            }
            Constant::Bool(b) => {
                self.output
                    .push_str(&format!("    mov {}, {}\n", dest, if *b { 1 } else { 0 }));
            }
            Constant::String(s) => {
                let label = format!("str_{}", self.string_counter);
                self.string_counter += 1;
                self.emit_string_data(&label, s);
                // For strings (pointers), use 64-bit register
                let reg = if dest == "eax" { "rax" } else { dest };
                self.output
                    .push_str(&format!("    lea {}, [{}]\n", reg, label));
            }
            Constant::Char(c) => {
                let reg = if dest == "eax" { "eax" } else { dest };
                self.output
                    .push_str(&format!("    mov {}, {}\n", reg, *c as i32));
            }
        }
    }

    fn emit_string_data(&mut self, label: &str, s: &str) {
        let bytes = self.escape_string(s);
        self.data_section.push_str(&format!("{} db ", label));

        for (i, b) in bytes.iter().enumerate() {
            if i > 0 {
                self.data_section.push_str(", ");
            }
            self.data_section.push_str(&format!("{}", b));
        }
        self.data_section.push_str(", 0\n");
    }

    fn escape_string(&self, s: &str) -> Vec<u8> {
        let mut result = Vec::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(&next) = chars.peek() {
                    match next {
                        'n' => {
                            result.push(10);
                            chars.next();
                        }
                        't' => {
                            result.push(9);
                            chars.next();
                        }
                        'r' => {
                            result.push(13);
                            chars.next();
                        }
                        '\\' => {
                            result.push(92);
                            chars.next();
                        }
                        '"' => {
                            result.push(34);
                            chars.next();
                        }
                        '\'' => {
                            result.push(39);
                            chars.next();
                        }
                        '0' => {
                            result.push(0);
                            chars.next();
                        }
                        _ => {
                            result.push(c as u8);
                        }
                    }
                }
            } else {
                result.push(c as u8);
            }
        }

        result
    }

    fn format_block_label(&self, id: &str) -> String {
        if id.starts_with("BB") {
            format!("BB_{}", id.trim_start_matches("BB"))
        } else {
            id.to_string()
        }
    }

    fn store_variable(&mut self, name: &str, src: &str, _is_pointer: bool) {
        if let Some(offset) = self.locals.get(name) {
            self.output
                .push_str(&format!("    mov [rbp + {}], {}\n", offset, src));
        } else if let Some(offset) = self.temps.get(name) {
            self.output
                .push_str(&format!("    mov [rbp + {}], {}\n", offset, src));
        }
    }
}
