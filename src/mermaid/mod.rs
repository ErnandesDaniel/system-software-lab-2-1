pub mod expressions;
pub mod statements;

use crate::ast::*;

pub struct MermaidGenerator {
    id_counter: usize,
}

impl MermaidGenerator {
    pub fn new() -> Self {
        Self { id_counter: 0 }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        let mut output = String::from("graph TD;\n");
        self.generate_program(program, &mut output, None);
        output
    }

    fn generate_program(
        &mut self,
        program: &Program,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"source\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        for item in &program.items {
            self.generate_source_item(item, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_source_item(
        &mut self,
        item: &SourceItem,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"source_item\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        match item {
            SourceItem::FuncDeclaration(f) => {
                self.generate_func_declaration(f, output, Some(&format!("N{}", id)));
            }
            SourceItem::FuncDefinition(f) => {
                self.generate_func_definition(f, output, Some(&format!("N{}", id)));
            }
        }
    }

    fn generate_func_declaration(
        &mut self,
        f: &FuncDeclaration,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"def\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_func_signature(&f.signature, output, Some(&format!("N{}", id)));
    }

    fn generate_func_definition(
        &mut self,
        f: &FuncDefinition,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"def\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_func_signature(&f.signature, output, Some(&format!("N{}", id)));

        for stmt in &f.body {
            self.generate_statement(stmt, output, Some(&format!("N{}", id)));
        }

        let end_id = self.next_id();
        output.push_str(&format!("N{}[\"end\"]\n", end_id));
        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, end_id));
        }
    }

    fn generate_func_signature(
        &mut self,
        sig: &FuncSignature,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"func_signature\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_identifier(&sig.name, output, Some(&format!("N{}", id)));

        let lparen_id = self.next_id();
        output.push_str(&format!("N{}[\"(\"]\n", lparen_id));
        output.push_str(&format!("N{} --> N{}\n", id, lparen_id));

        if let Some(params) = &sig.parameters {
            let mut first = true;
            for arg in params {
                if first {
                    first = false;
                } else {
                    let comma_id = self.next_id();
                    output.push_str(&format!("N{}[\",\"]\n", comma_id));
                    output.push_str(&format!("N{} --> N{}\n", id, comma_id));
                }
                self.generate_arg(arg, output, Some(&format!("N{}", id)));
            }
        }

        let rparen_id = self.next_id();
        output.push_str(&format!("N{}[\")\"]\n", rparen_id));
        output.push_str(&format!("N{} --> N{}\n", id, rparen_id));

        if let Some(ret) = &sig.return_type {
            self.generate_type_ref(ret, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_arg(&mut self, arg: &Arg, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"identifier\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_identifier(&arg.name, output, Some(&format!("N{}", id)));

        if let Some(ty) = &arg.ty {
            let of_id = self.next_id();
            output.push_str(&format!("N{}[\"of\"]\n", of_id));
            output.push_str(&format!("N{} --> N{}\n", id, of_id));
            self.generate_type_ref(ty, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_type_ref(&mut self, ty: &TypeRef, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        let label = match ty {
            TypeRef::BuiltinType(b) => format!("{:?}", b),
            TypeRef::Custom(id) => id.name.clone(),
            TypeRef::Array {
                element_type: _,
                size,
                ..
            } => format!("array[{}]", size),
        };
        output.push_str(&format!("N{}[\"type_ref: {}\"]\n", id, label));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        if let TypeRef::Array { element_type, .. } = ty {
            self.generate_type_ref(element_type, output, Some(&format!("N{}", id)));
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }
}
