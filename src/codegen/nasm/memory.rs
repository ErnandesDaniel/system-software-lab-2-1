use crate::ir::types::{IrInstruction, IrOperand, IrType};

use crate::codegen::nasm::AsmGenerator;

impl AsmGenerator {
    pub fn generate_store(&mut self, inst: &IrInstruction) {
        if inst.operands.len() >= 3 {
            let base = &inst.operands[0];
            let offset = &inst.operands[1];
            let value = &inst.operands[2];

            if let IrOperand::Variable(name, _) = base {
                if let IrOperand::Constant(crate::ir::Constant::Int(off)) = offset {
                    if inst.operands.len() == 4 {
                        let index = &inst.operands[3];
                        let elem_stride = Self::array_elem_stride(inst);
                        let val_is_ptr = value.get_type().is_pointer();
                        let val_reg = if val_is_ptr { "rax" } else { "eax" };
                        self.load_operand(value, val_reg, val_is_ptr);
                        if let IrOperand::Constant(crate::ir::Constant::Int(idx_val)) = index {
                            let total_off = *off as i64 + idx_val * elem_stride;
                            self.gen_lea_base(name, "rcx");
                            self.output.push_str(&format!("    mov [rcx + {total_off}], {val_reg}\n"));
                        } else {
                            self.load_operand(index, "ebx", false);
                            self.gen_lea_base(name, "rcx");
                            if *off != 0 {
                                self.output.push_str(&format!("    add rcx, {off}\n"));
                            }
                            if matches!(elem_stride, 1 | 2 | 4 | 8) {
                                self.output.push_str(&format!("    mov [rcx + rbx * {elem_stride}], {val_reg}\n"));
                            } else {
                                self.output.push_str(&format!("    imul ebx, {elem_stride}\n"));
                                self.output.push_str("    add rcx, rbx\n");
                                self.output.push_str(&format!("    mov [rcx], {val_reg}\n"));
                            }
                        }
                    } else {
                        let val_is_ptr = value.get_type().is_pointer();
                        let val_reg = if val_is_ptr { "rax" } else { "eax" };
                        self.load_operand(value, val_reg, val_is_ptr);
                        if let Some(local_off) = self.locals.get(name) {
                            let addr = local_off + *off as i32;
                            self.output.push_str(&format!("    mov [rbp + {addr}], {val_reg}\n"));
                        } else if let Some(param_idx) = self.param_registers.iter().position(|r| r == name) {
                            let reg = self.get_param_register(param_idx, true);
                            if *off == 0 {
                                self.output.push_str(&format!("    mov [{reg}], {val_reg}\n"));
                            } else {
                                self.output.push_str(&format!("    mov [{reg} + {off}], {val_reg}\n"));
                            }
                        } else if *off == 0 {
                            self.output.push_str(&format!("    mov [rel {name}], {val_reg}\n"));
                        } else {
                            self.output.push_str(&format!("    mov [rel {name} + {off}], {val_reg}\n"));
                        }
                    }
                }
            }
        }
    }

    pub fn generate_load(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(_operand0)) = (&inst.result, inst.operands.first()) {
            let result_reg = if self.load_result_reg(inst) == "rax" { "rax" } else { "eax" };
            if inst.operands.len() == 1 {
                if let IrOperand::Variable(name, _) = &inst.operands[0] {
                    if let Some(local_off) = self.locals.get(name) {
                        self.output.push_str(&format!("    mov {result_reg}, [rbp + {local_off}]\n"));
                    } else {
                        self.output.push_str(&format!("    mov {result_reg}, [rel {name}]\n"));
                    }
                    self.store_variable(result, result_reg, result_reg == "rax");
                }
            } else if inst.operands.len() == 4 {
                if let IrOperand::Variable(name, _) = &inst.operands[0] {
                    if let (
                        IrOperand::Constant(crate::ir::Constant::Int(field_off)),
                        IrOperand::Constant(crate::ir::Constant::Int(elem_sz)),
                    ) = (&inst.operands[1], &inst.operands[3])
                    {
                        self.load_operand(&inst.operands[2], "ebx", false);
                        self.gen_lea_base(name, "rax");
                        if *field_off != 0 {
                            self.output.push_str(&format!("    add rax, {field_off}\n"));
                        }
                        if *elem_sz != 1 {
                            self.output.push_str(&format!("    imul ebx, {elem_sz}\n"));
                        }
                        self.output.push_str("    add rax, rbx\n");
                        self.output.push_str(&format!("    mov {result_reg}, [rax]\n"));
                        self.store_variable(result, result_reg, result_reg == "rax");
                    }
                }
            } else if inst.operands.len() == 3 {
                if let IrOperand::Variable(name, _) = &inst.operands[0] {
                    if let IrOperand::Constant(crate::ir::Constant::Int(off)) = &inst.operands[1] {
                        let elem_stride = Self::array_elem_stride(inst);
                        self.load_operand(&inst.operands[2], "ebx", false);
                        self.gen_lea_base(name, "rax");
                        if *off != 0 {
                            self.output.push_str(&format!("    add rax, {off}\n"));
                        }
                        if elem_stride == 8 {
                            self.output.push_str(&format!("    mov {result_reg}, [rax + rbx * 8]\n"));
                        } else {
                            self.output.push_str(&format!("    mov {result_reg}, [rax + rbx * 4]\n"));
                        }
                        self.store_variable(result, result_reg, result_reg == "rax");
                    }
                }
            } else if inst.operands.len() == 2 {
                let first = &inst.operands[0];
                let second = &inst.operands[1];
                if let IrOperand::Constant(c) = second {
                    if let IrOperand::Variable(name, _) = first {
                        if let crate::ir::Constant::Int(offset) = c {
                            if let Some(local_off) = self.locals.get(name) {
                                let addr = local_off + *offset as i32;
                                self.output.push_str(&format!("    mov {result_reg}, [rbp + {addr}]\n"));
                            } else if let Some(param_idx) = self.param_registers.iter().position(|r| r == name) {
                                let reg = self.get_param_register(param_idx, true);
                                if *offset == 0 {
                                    self.output.push_str(&format!("    mov {result_reg}, [{reg}]\n"));
                                } else {
                                    self.output.push_str(&format!("    mov {result_reg}, [{reg} + {offset}]\n"));
                                }
                            } else if *offset == 0 {
                                self.output.push_str(&format!("    mov {result_reg}, [rel {name}]\n"));
                            } else {
                                self.output.push_str(&format!("    mov {result_reg}, [rel {name} + {offset}]\n"));
                            }
                            self.store_variable(result, result_reg, result_reg == "rax");
                        }
                    }
                } else {
                    let elem_stride = Self::array_elem_stride(inst);
                    if let IrOperand::Variable(name, _) = first {
                        self.gen_lea_base(name, "rax");
                    } else {
                        self.load_operand(first, "rax", true);
                    }
                    self.load_operand(second, "ebx", false);
                    if elem_stride == 8 {
                        self.output.push_str(&format!("    mov {result_reg}, [rax + rbx * 8]\n"));
                    } else {
                        self.output.push_str(&format!("    mov {result_reg}, [rax + rbx * 4]\n"));
                    }
                    self.store_variable(result, result_reg, result_reg == "rax");
                }
            }
        }
    }

    pub fn generate_slice(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(base), Some(start)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            match base.get_type() {
                IrType::String => {
                    self.load_operand(base, "rcx", true);
                    self.load_operand(start, "edx", false);
                    self.output.push_str("    lea rax, [rcx + rdx]\n");
                    self.store_variable(result, "rax", true);
                }
                IrType::Array(elem_type, _size) => {
                    self.load_operand(base, "rax", true);
                    self.load_operand(start, "ebx", false);
                    let elem_size = elem_type.size() as i32;
                    self.output.push_str(&format!("    imul ebx, ebx, {elem_size}\n"));
                    self.output.push_str("    add rax, rbx\n");
                    self.store_variable(result, "rax", true);
                }
                _ => {}
            }
        }
    }

    pub fn generate_str_get_byte(&mut self, inst: &IrInstruction) {
        if let (Some(ref result), Some(str_op), Some(idx_op)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(str_op, "rcx", true);
            self.load_operand(idx_op, "edx", false);
            self.output.push_str("    movzx eax, byte [rcx + rdx]\n");
            self.store_variable(result, "eax", false);
        }
    }

    pub fn generate_str_set_byte(&mut self, inst: &IrInstruction) {
        if let (Some(str_op), Some(idx_op), Some(val_op)) = (inst.operands.first(), inst.operands.get(1), inst.operands.get(2)) {
            self.load_operand(str_op, "rcx", true);
            self.load_operand(idx_op, "edx", false);
            self.load_operand(val_op, "r8d", false);
            self.output.push_str("    mov [rcx + rdx], r8b\n");
        }
    }

    pub(crate) fn store_reg_for_type(&self, ty: &IrType) -> &'static str {
        if ty.is_pointer() { "rax" } else { "eax" }
    }

    pub(crate) fn load_result_reg(&self, inst: &IrInstruction) -> &'static str {
        inst.result_type.as_ref().map_or("eax", |t| self.store_reg_for_type(t))
    }

    pub(crate) fn array_elem_stride(inst: &IrInstruction) -> i64 {
        if let Some(IrOperand::Variable(_, ty)) = inst.operands.first() {
            if let IrType::Array(elem_type, _) = ty {
                return elem_type.size() as i64;
            }
        }
        4
    }

    pub(crate) fn gen_lea_base(&mut self, name: &str, reg: &str) {
        if let Some(local_off) = self.locals.get(name) {
            self.output.push_str(&format!("    lea {reg}, [rbp + {local_off}]\n"));
        } else {
            self.output.push_str(&format!("    lea {reg}, [rel {name}]\n"));
        }
    }
}
