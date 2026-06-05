use crate::ast::{BuiltinType, Expr, TypeRef};
use crate::ir::IrType;
use crate::ir_generator::unescape_string;

use super::IrGenerator;

impl IrGenerator {
    pub fn convert_type(&self, ty: &TypeRef) -> IrType {
        match ty {
            TypeRef::BuiltinType(bt) => match bt {
                BuiltinType::Bool => IrType::Bool,
                BuiltinType::Byte => IrType::Byte,
                BuiltinType::Int => IrType::Int,
                BuiltinType::Uint => IrType::Uint,
                BuiltinType::Long => IrType::Long,
                BuiltinType::Ulong => IrType::Ulong,
                BuiltinType::Char => IrType::Char,
                BuiltinType::String => IrType::String,
            },
            TypeRef::Custom(id) => {
                if let Some(fields) = self.symbols.struct_fields.get(&id.name) {
                    let total_size: usize = fields.iter().map(|(_, ty, _)| ty.size() as usize).sum();
                    let typed_fields: Vec<(String, IrType)> =
                        fields.iter().map(|(n, ty, _)| (n.clone(), ty.clone())).collect();
                    IrType::Struct {
                        name: id.name.clone(),
                        fields: typed_fields,
                        size: total_size,
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
                crate::ast::Literal::Hex(v) => Some(crate::ir::Constant::Int(*v as i64)),
                crate::ast::Literal::Bits(v) => Some(crate::ir::Constant::Int(*v as i64)),
                crate::ast::Literal::Str(s) => Some(crate::ir::Constant::String(unescape_string(s))),
                crate::ast::Literal::Char(c) => Some(crate::ir::Constant::Char(*c as u8)),
                crate::ast::Literal::Bool(b) => Some(crate::ir::Constant::Bool(*b)),
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

    pub fn find_field_offset_for_array(&self, base: &str, field: &str) -> usize {
        let struct_name = self
            .symbols
            .global_struct_type_names
            .get(base)
            .or_else(|| self.symbols.local_struct_types.get(base));
        if let Some(struct_name) = struct_name {
            if let Some(fields) = self.symbols.struct_fields.get(struct_name) {
                for (fname, _, offset) in fields {
                    if fname == field {
                        return *offset;
                    }
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
                return fields.iter().map(|(_, ty, _)| ty.size() as usize).sum();
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

    /// Resolve a chain of field accesses like a.b.c → (base_name, total_byte_offset).
    /// Uses the struct type name stored in global_struct_type_names or local_struct_types
    /// to traverse nested struct fields.
    pub fn resolve_field_chain(&self, expr: &Expr) -> (String, usize) {
        match expr {
            Expr::FieldAccess(base, field, _) => {
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

    /// Resolve struct field info for a field access expression.
    /// Returns (struct_type, total_offset, field_type).
    pub fn resolve_field_info(
        &self,
        base_name: &str,
        _base: &Expr,
        field: &crate::ast::Identifier,
        base_offset: usize,
    ) -> (IrType, usize, IrType) {
        let struct_name = self
            .symbols
            .local_struct_types
            .get(base_name)
            .map(String::as_str)
            .or_else(|| self.symbols.global_struct_type_names.get(base_name).map(String::as_str))
            .unwrap_or(base_name);

        let (field_offset, field_type) = self
            .symbols
            .struct_fields
            .get(struct_name)
            .and_then(|fields| fields.iter().find(|(n, _, _)| n == &field.name))
            .map(|(_, ty, o)| (*o, ty.clone()))
            .unwrap_or((0, IrType::Int));

        let struct_size = self
            .symbols
            .struct_fields
            .get(struct_name)
            .map(|fields| fields.iter().map(|(_, ty, _)| ty.size() as usize).sum::<usize>())
            .unwrap_or(4);

        let typed_fields: Vec<(String, IrType)> = self
            .symbols
            .struct_fields
            .get(struct_name)
            .map(|fields| fields.iter().map(|(n, ty, _)| (n.clone(), ty.clone())).collect())
            .unwrap_or_default();

        (
            IrType::Struct {
                name: struct_name.to_string(),
                fields: typed_fields,
                size: struct_size,
            },
            base_offset + field_offset,
            field_type,
        )
    }
}
