use crate::ir::IrType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructFieldInfo {
    pub name: String,
    pub ty: IrType,
    pub byte_offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructLayout {
    pub name: String,
    pub fields: Vec<StructFieldInfo>,
    pub total_size: usize,
}

impl StructLayout {
    pub fn field_offset(&self, field_name: &str) -> Option<usize> {
        self.fields.iter().find(|f| f.name == field_name).map(|f| f.byte_offset)
    }

    pub fn field_type(&self, field_name: &str) -> Option<IrType> {
        self.fields.iter().find(|f| f.name == field_name).map(|f| f.ty.clone())
    }

    pub fn field_index(&self, field_name: &str) -> Option<usize> {
        self.fields.iter().position(|f| f.name == field_name)
    }

    pub fn size_in_slots(&self) -> usize {
        self.total_size.div_ceil(4)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutDatabase {
    pub structs: HashMap<String, StructLayout>,
}

impl LayoutDatabase {
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
        }
    }

    pub fn register_struct(&mut self, name: &str, fields: &[(String, IrType)]) {
        let mut struct_fields = Vec::new();
        let mut offset: usize = 0;
        for (fname, fty) in fields {
            let size = fty.size() as usize;
            struct_fields.push(StructFieldInfo {
                name: fname.clone(),
                ty: fty.clone(),
                byte_offset: offset,
            });
            offset += size;
        }
        self.structs.insert(
            name.to_string(),
            StructLayout {
                name: name.to_string(),
                fields: struct_fields,
                total_size: offset,
            },
        );
    }

    pub fn get(&self, name: &str) -> Option<&StructLayout> {
        self.structs.get(name)
    }

    pub fn resolve_field_chain(&self, base_struct: &str, field: &str) -> Option<(usize, IrType)> {
        let layout = self.structs.get(base_struct)?;
        let info = layout.fields.iter().find(|f| f.name == field)?;
        Some((info.byte_offset, info.ty.clone()))
    }

    pub fn resolve_field_type(&self, base_struct: &str, field: &str) -> IrType {
        self.resolve_field_chain(base_struct, field)
            .map(|(_, ty)| ty)
            .unwrap_or(IrType::Int)
    }

    pub fn has_struct(&self, name: &str) -> bool {
        self.structs.contains_key(name)
    }
}
