use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::IrType;

impl AsmGenerator {
    pub fn generate_globals_asm(globals: &[crate::ir::IrGlobal]) -> String {
        let mut output = String::new();
        if globals.is_empty() {
            return output;
        }
        for global in globals {
            output.push_str(&format!("global {}\n", global.name));
            match &global.ty {
                IrType::Int | IrType::Bool => {
                    let val = match &global.initializer {
                        Some(crate::ir::Constant::Int(v)) => *v,
                        _ => 0,
                    };
                    output.push_str(&format!("{} dd {}\n", global.name, val));
                }
                IrType::String => {
                    if let Some(crate::ir::Constant::String(s)) = &global.initializer {
                        let slabel = format!("{}_str", global.name);
                        let bytes: Vec<u8> = s.bytes().collect();
                        output.push_str(&format!("{slabel} db "));
                        if bytes.is_empty() {
                            output.push('0');
                        } else {
                            for (j, b) in bytes.iter().enumerate() {
                                if j > 0 {
                                    output.push_str(", ");
                                }
                                output.push_str(&format!("{b}"));
                            }
                        }
                        output.push_str(", 0\n");
                        output.push_str(&format!("{} dq {}\n", global.name, slabel));
                    } else {
                        output.push_str(&format!("{} dq 0\n", global.name));
                    }
                }
                IrType::Array(elem_type, size) => {
                    let label = global.name.clone();
                    match elem_type.as_ref() {
                        IrType::Int => {
                            output.push_str(&format!("{label} dd "));
                            if let Some(crate::ir::Constant::Array(elems)) = &global.initializer {
                                for (i, elem) in elems.iter().enumerate() {
                                    if i > 0 {
                                        output.push_str(", ");
                                    }
                                    if let crate::ir::Constant::Int(v) = elem {
                                        output.push_str(&format!("{v}"));
                                    } else {
                                        output.push('0');
                                    }
                                }
                                for i in elems.len()..*size {
                                    if i > 0 || !elems.is_empty() {
                                        output.push_str(", ");
                                    }
                                    output.push('0');
                                }
                            } else {
                                for i in 0..*size {
                                    if i > 0 {
                                        output.push_str(", ");
                                    }
                                    output.push('0');
                                }
                            }
                            output.push('\n');
                        }
                        IrType::String => {
                            let init_elems = match &global.initializer {
                                Some(crate::ir::Constant::Array(elems)) => elems.as_slice(),
                                _ => &[],
                            };
                            for i in 0..*size {
                                let slabel = format!("{}_{}", global.name, i);
                                let s = init_elems
                                    .get(i)
                                    .and_then(|e| {
                                        if let crate::ir::Constant::String(s) = e {
                                            Some(s.as_str())
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or("");
                                output.push_str(&format!("{slabel} db "));
                                let bytes: Vec<u8> = s.bytes().collect();
                                if bytes.is_empty() {
                                    output.push('0');
                                } else {
                                    for (j, b) in bytes.iter().enumerate() {
                                        if j > 0 {
                                            output.push_str(", ");
                                        }
                                        output.push_str(&format!("{b}"));
                                    }
                                    output.push_str(", 0");
                                }
                                output.push('\n');
                            }
                            output.push_str(&format!("{label} dq "));
                            for i in 0..*size {
                                if i > 0 {
                                    output.push_str(", ");
                                }
                                output.push_str(&format!("{}_{}", global.name, i));
                            }
                            output.push('\n');
                        }
                        _ => {
                            let elem_size = elem_type.size() as usize;
                            output.push_str(&format!("{label} times {} db 0\n", elem_size * size));
                        }
                    }
                }
                _ => {
                    let size = global.ty.size() as usize;
                    output.push_str(&format!("{} times {} db 0\n", global.name, size));
                }
            }
        }
        output
    }
}
