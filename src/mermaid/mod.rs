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

    pub fn generate_function(&mut self, func: &FuncDefinition) -> String {
        let mut output = String::from("graph TD;\n");
        let id = self.next_id();
        output.push_str(&format!(
            "N{}[\"function: {}\"]\n",
            id, func.signature.name.name
        ));

        for stmt in &func.body {
            self.generate_statement(stmt, &mut output, Some(&format!("N{}", id)));
        }

        output
    }

    fn next_id(&mut self) -> usize {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }
}
