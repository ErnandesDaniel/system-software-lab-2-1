use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::{Constant, IrInstruction, IrOperand, IrType};
use crate::ir::IrOpcode;

impl AsmGenerator {
    pub fn emit_store(&mut self, inst: &IrInstruction) {
        if inst.operands.len() < 3 {
            return;
        }
        let base = &inst.operands[0];
        let offset = &inst.operands[1];
        let value = &inst.operands[2];

        let off = match offset {
            IrOperand::Constant(Constant::Int(v)) => *v,
            _ => 0,
        };
        let (base_name, base_type) = match base {
            IrOperand::Variable(n, ty) => (n.clone(), ty.clone()),
            _ => return,
        };

        let is_struct_ptr = matches!(base_type, IrType::Struct { .. });

        if inst.operands.len() >= 4 {
            let index = &inst.operands[3];
            let elem_stride = Self::elem_stride_for(inst);
            let val_wide = AsmGenerator::is_wide_type(&value.get_type());
            let val_reg = self.alloc_scratch(val_wide);
            self.load_operand(value, val_reg);

            let base_reg = self.alloc_scratch(true);
            let base_mem = self.mem_for(&base_name);
            if is_struct_ptr {
                self.line(&format!("mov {base_reg}, {base_mem}"));
            } else {
                self.line(&format!("lea {base_reg}, {base_mem}"));
            }

            match index {
                IrOperand::Constant(Constant::Int(idx_val)) => {
                    let total = off + idx_val * elem_stride;
                    self.line(&format!("mov [{base_reg} + {total}], {val_reg}"));
                }
                _ => {
                    let idx_reg = self.alloc_scratch(true);
                    self.load_operand(index, idx_reg);
                    if off != 0 {
                        self.line(&format!("add {base_reg}, {off}"));
                    }
                    if matches!(elem_stride, 1 | 2 | 4 | 8) {
                        self.line(&format!("mov [{base_reg} + {idx_reg}*{elem_stride}], {val_reg}"));
                    } else {
                        self.line(&format!("imul {idx_reg}, {elem_stride}"));
                        self.line(&format!("add {base_reg}, {idx_reg}"));
                        self.line(&format!("mov [{base_reg}], {val_reg}"));
                    }
                }
            }
            return;
        }

        let val_wide = AsmGenerator::is_wide_type(&value.get_type());
        let val_reg = self.alloc_scratch(val_wide);
        self.load_operand(value, val_reg);
        let base_mem = self.mem_for(&base_name);
        if is_struct_ptr {
            let ptr_reg = self.alloc_scratch(true);
            self.line(&format!("mov {ptr_reg}, {base_mem}"));
            self.line(&format!("mov [{ptr_reg} + {off}], {val_reg}"));
        } else if off == 0 {
            self.line(&format!("mov {base_mem}, {val_reg}"));
        } else {
            self.line(&format!("lea rcx, {base_mem}"));
            self.line(&format!("mov [rcx + {off}], {val_reg}"));
        }
    }

    pub fn emit_load(&mut self, inst: &IrInstruction) {
        let result = match &inst.result {
            Some(r) => r.clone(),
            _ => return,
        };
        let result_ty = inst.result_type.as_ref().cloned().unwrap_or(IrType::Int);
        let result_wide = AsmGenerator::is_wide_type(&result_ty);

        let is_struct_base = inst.operands.first().map_or(false, |op| {
            matches!(op, IrOperand::Variable(_, IrType::Struct { .. }))
        });

        match inst.operands.len() {
            1 => {
                if let IrOperand::Variable(_name, _) = &inst.operands[0] {
                    let reg = self.alloc_scratch(result_wide);
                    self.load_operand(&inst.operands[0], reg);
                    self.store_result(&result, reg, &result_ty);
                }
            }
            2 => {
                if let IrOperand::Constant(Constant::Int(off)) = &inst.operands[1] {
                    let (base_name, _) = match &inst.operands[0] {
                        IrOperand::Variable(n, _ty) => (n.clone(), ()),
                        _ => return,
                    };
                    let reg = self.alloc_scratch(result_wide);
                    let mem = self.mem_for(&base_name);
                    let is_ptr_type = matches!(result_ty, IrType::Array(_, _) | IrType::Struct { .. });

                    if is_struct_base {
                        let base_reg = self.alloc_scratch(true);
                        self.line(&format!("mov {base_reg}, {mem}"));
                        if *off == 0 {
                            if is_ptr_type {
                                self.line(&format!("lea {reg}, [{base_reg}]"));
                            } else {
                                self.line(&format!("mov {reg}, [{base_reg}]"));
                            }
                        } else if is_ptr_type {
                            self.line(&format!("lea {reg}, [{base_reg} + {off}]"));
                        } else {
                            self.line(&format!("mov {reg}, [{base_reg} + {off}]"));
                        }
                    } else if *off == 0 {
                        if is_ptr_type {
                            self.line(&format!("lea {reg}, {mem}"));
                        } else {
                            self.line(&format!("mov {reg}, {mem}"));
                        }
                    } else {
                        self.line(&format!("lea rcx, {mem}"));
                        if is_ptr_type {
                            self.line(&format!("lea {reg}, [rcx + {off}]"));
                        } else {
                            self.line(&format!("mov {reg}, [rcx + {off}]"));
                        }
                    }
                    self.store_result(&result, reg, &result_ty);
                } else {
                    let (array_name, base_ty) = match &inst.operands[0] {
                        IrOperand::Variable(n, ty) => (n.clone(), ty.clone()),
                        _ => return,
                    };
                    let is_arr_struct = matches!(base_ty, IrType::Struct { .. });
                    let elem_stride = Self::elem_stride_for(inst);
                    let idx_reg = self.alloc_scratch(true);
                    self.load_operand(&inst.operands[1], idx_reg);
                    let res_reg = self.alloc_scratch(result_wide);
                    let base_reg = self.alloc_scratch(true);
                    let mem = self.mem_for(&array_name);
                    if is_arr_struct {
                        self.line(&format!("mov {base_reg}, {mem}"));
                    } else {
                        self.line(&format!("lea {base_reg}, {mem}"));
                    }
                    if matches!(elem_stride, 1 | 2 | 4 | 8) {
                        self.line(&format!("mov {res_reg}, [{base_reg} + {idx_reg}*{elem_stride}]"));
                    } else {
                        self.line(&format!("imul {idx_reg}, {elem_stride}"));
                        self.line(&format!("add {base_reg}, {idx_reg}"));
                        self.line(&format!("mov {res_reg}, [{base_reg}]"));
                    }
                    self.store_result(&result, res_reg, &result_ty);
                }
            }
            3 | 4 => {
                let (base_name, base_ty) = match &inst.operands[0] {
                    IrOperand::Variable(n, ty) => (n.clone(), ty.clone()),
                    _ => return,
                };
                let is_arr_struct = matches!(base_ty, IrType::Struct { .. });
                let off = match &inst.operands[1] {
                    IrOperand::Constant(Constant::Int(v)) => *v,
                    _ => 0,
                };
                let elem_stride = Self::elem_stride_for(inst);
                let idx_reg = self.alloc_scratch(true);
                self.load_operand(&inst.operands[2], idx_reg);
                let res_reg = self.alloc_scratch(result_wide);
                let base_reg = self.alloc_scratch(true);
                let mem = self.mem_for(&base_name);
                if is_arr_struct {
                    self.line(&format!("mov {base_reg}, {mem}"));
                } else {
                    self.line(&format!("lea {base_reg}, {mem}"));
                }
                if off != 0 {
                    self.line(&format!("add {base_reg}, {off}"));
                }
                if elem_stride != 1 {
                    self.line(&format!("imul {idx_reg}, {elem_stride}"));
                }
                self.line(&format!("mov {res_reg}, [{base_reg} + {idx_reg}]"));
                self.store_result(&result, res_reg, &result_ty);
            }
            _ => {}
        }
    }

    pub fn emit_slice(&mut self, inst: &IrInstruction) {
        if let (Some(ref result), Some(base), Some(start)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            match base.get_type() {
                IrType::String => {
                    self.load_operand(base, "rcx");
                    self.load_operand(start, "edx");
                    self.line("lea rax, [rcx + rdx]");
                    self.store_result(result, "rax", &IrType::String);
                }
                IrType::Array(elem_type, _) => {
                    self.load_operand(base, "rax");
                    self.load_operand(start, "ebx");
                    let elem_size = elem_type.size() as i64;
                    self.line(&format!("imul rbx, rbx, {elem_size}"));
                    self.line("add rax, rbx");
                    self.store_result(result, "rax", &IrType::Array(elem_type.clone(), 0));
                }
                _ => {}
            }
        }
    }

    pub fn emit_str_get_byte(&mut self, inst: &IrInstruction) {
        if let (Some(ref result), Some(str_op), Some(idx_op)) =
            (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            self.load_operand(str_op, "rcx");
            self.load_operand(idx_op, "edx");
            self.line("movzx eax, byte [rcx + rdx]");
            self.store_result(result, "eax", &IrType::Int);
        }
    }

    pub fn emit_str_set_byte(&mut self, inst: &IrInstruction) {
        if let (Some(str_op), Some(idx_op), Some(val_op)) =
            (inst.operands.first(), inst.operands.get(1), inst.operands.get(2))
        {
            self.load_operand(str_op, "rcx");
            self.load_operand(idx_op, "edx");
            self.load_operand(val_op, "r8d");
            self.line("mov [rcx + rdx], r8b");
        }
    }

    fn elem_stride_for(inst: &IrInstruction) -> i64 {
        // 5th operand (arr[i].field = v or s.f[i] = v): element size
        if let Some(IrOperand::Constant(Constant::Int(stride))) = inst.operands.get(4) {
            return *stride;
        }
        // 4th operand (s.f[i] or arr[i].f): element size for Load
        // NOTE: only check for Load — for Store the 4th operand is the index, not elem_size
        if inst.opcode == IrOpcode::Load {
            if let Some(IrOperand::Constant(Constant::Int(stride))) = inst.operands.get(3) {
                return *stride;
            }
        }
        if let Some(IrOperand::Variable(_, ty)) = inst.operands.first() {
            match ty {
                IrType::Array(elem, _) => return elem.size() as i64,
                IrType::Struct { size, .. } if inst.operands.len() >= 4 => return *size as i64,
                IrType::Struct { size, .. } => return *size as i64,
                _ => {}
            }
        }
        4
    }
}
