use crate::ast::{BuiltinType, Expr, Identifier, TypeRef};
use crate::ir::IrType;

use super::IrGenerator;

impl IrGenerator {
    pub fn get_ident_type(&self, id: &Identifier) -> IrType {
        self.symbols.get_type(&id.name)
    }

    pub fn convert_type(&self, ty: &TypeRef) -> IrType {
        match ty {
            TypeRef::BuiltinType(bt) => match bt {
                BuiltinType::Bool => IrType::Bool,
                BuiltinType::Byte
                | BuiltinType::Int
                | BuiltinType::Uint
                | BuiltinType::Long
                | BuiltinType::Ulong
                | BuiltinType::Char => IrType::Int,
                BuiltinType::String => IrType::String,
            },
            TypeRef::Custom(id) => {
                if let Some(fields) = self.symbols.struct_fields.get(&id.name) {
                    if let Some(last) = fields.last() {
                        let size = last.2 + last.1.size() as usize;
                        IrType::Array(Box::new(IrType::Int), size / 4)
                    } else {
                        IrType::Int
                    }
                } else {
                    IrType::Int
                }
            }
            TypeRef::Array { element_type, size, .. } => {
                IrType::Array(Box::new(self.convert_type(element_type)), *size as usize)
            }
            TypeRef::Function {
                params, return_type, ..
            } => {
                let p: Vec<IrType> = params.iter().map(|t| self.convert_type(t)).collect();
                IrType::Function(p, Box::new(self.convert_type(return_type)))
            }
        }
    }

    pub fn expr_to_constant(&self, expr: &Expr) -> Option<crate::ir::Constant> {
        match expr {
            Expr::Literal(lit, _) => match lit {
                crate::ast::Literal::Dec(v) => Some(crate::ir::Constant::Int(*v as i64)),
                crate::ast::Literal::Str(s) => Some(crate::ir::Constant::String(s.clone())),
                crate::ast::Literal::Char(c) => Some(crate::ir::Constant::Char(*c as u8)),
                crate::ast::Literal::Bool(b) => Some(crate::ir::Constant::Int(i64::from(*b))),
                _ => None,
            },
            Expr::ArrayLiteral(elements, _) => {
                let constants: Vec<crate::ir::Constant> = elements
                    .iter()
                    .map(|e| self.expr_to_constant(e))
                    .collect::<Option<_>>()?;
                Some(crate::ir::Constant::Array(constants))
            }
            _ => None,
        }
    }

    pub fn find_field_offset_for_array(&self, _base: &str, field: &str) -> usize {
        for fields in self.symbols.struct_fields.values() {
            for (fname, _, offset) in fields {
                if fname == field {
                    return *offset;
                }
            }
        }
        0
    }

    pub fn struct_size_for_var(&self, base: &str) -> usize {
        let struct_name = self
            .symbols
            .global_struct_type_names
            .get(base)
            .or_else(|| self.symbols.local_struct_types.get(base));
        if let Some(struct_name) = struct_name {
            if let Some(fields) = self.symbols.struct_fields.get(struct_name) {
                if let Some((_, last_type, last_offset)) = fields.last() {
                    return last_offset + last_type.size() as usize;
                }
            }
        }
        4
    }

    pub fn find_field_type_for_var(&self, base: &str, field: &str) -> IrType {
        let struct_name = self
            .symbols
            .global_struct_type_names
            .get(base)
            .or_else(|| self.symbols.local_struct_types.get(base));
        if let Some(struct_name) = struct_name {
            if let Some(fields) = self.symbols.struct_fields.get(struct_name) {
                for (fname, ftype, _) in fields {
                    if fname == field {
                        return ftype.clone();
                    }
                }
            }
        }
        IrType::Int
    }

    pub fn resolve_field_chain(&self, expr: &Expr) -> (String, usize) {
        match expr {
            Expr::FieldAccess(base, field) => {
                let (base_name, base_offset) = self.resolve_field_chain(base);
                let struct_name = self
                    .symbols
                    .local_struct_types
                    .get(&base_name)
                    .map(String::as_str)
                    .or_else(|| {
                        self.symbols
                            .global_struct_type_names
                            .get(&base_name)
                            .map(String::as_str)
                    })
                    .unwrap_or(&base_name);
                let field_offset = self
                    .symbols
                    .struct_fields
                    .get(struct_name)
                    .and_then(|fields| fields.iter().find(|(n, _, _)| n == &field.name))
                    .map_or(0, |(_, _, o)| *o);
                (base_name, base_offset + field_offset)
            }
            Expr::Identifier(id) => (id.name.clone(), 0),
            _ => (String::new(), 0),
        }
    }
}
