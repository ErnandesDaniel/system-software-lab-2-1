use crate::ast::*;
use crate::ir::*;
use crate::stdlib::StdLib;
use std::collections::{HashMap, HashSet};

mod expressions;
mod statements;

#[derive(Debug, Clone)]
pub struct CoroutineInfo {
    pub name: String,
    pub scheduler: String,
    pub stack_size: usize,
}

pub struct IrGenerator {
    pub temp_counter: usize,
    pub block_counter: usize,
    pub locals: HashMap<String, IrLocal>,
    pub declared_vars: HashSet<String>,
    pub used_functions: Vec<String>,
    pub loop_exit_stack: Vec<String>,
    pub loop_depth: usize,
    pub thread_functions: HashSet<String>,
    pub scheduler_type: HashMap<String, String>,
    pub external_functions: HashSet<String>,
    pub coroutines: HashMap<String, CoroutineInfo>,
    pub current_coroutine_id: usize,
}

impl IrGenerator {
    pub fn new() -> Self {
        Self {
            temp_counter: 0,
            block_counter: 0,
            locals: HashMap::new(),
            declared_vars: HashSet::new(),
            used_functions: Vec::new(),
            loop_exit_stack: Vec::new(),
            loop_depth: 0,
            thread_functions: HashSet::new(),
            scheduler_type: HashMap::new(),
            external_functions: HashSet::new(),
            coroutines: HashMap::new(),
            current_coroutine_id: 0,
        }
    }

    pub fn is_thread_function(&self, name: &str) -> bool {
        self.thread_functions.contains(name)
    }

    pub fn get_scheduler(&self, name: &str) -> String {
        self.scheduler_type
            .get(name)
            .cloned()
            .unwrap_or_else(|| "FCFS".to_string())
    }

    pub fn is_external_function(&self, name: &str) -> bool {
        // Function is external if:
        // 1. It's declared as extern def in the source code
        // 2. It's a stdlib function (C runtime)
        self.external_functions.contains(name) || StdLib::is_stdlib(name)
    }

    pub fn should_insert_yields(&self, func_name: &str) -> bool {
        self.thread_functions.contains(func_name) && !self.is_external_function(func_name)
    }

    pub fn generate_temp(&mut self) -> String {
        let temp = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    pub fn generate_block_id(&mut self) -> String {
        let id = format!("BB{}", self.block_counter);
        self.block_counter += 1;
        id
    }

    pub fn generate(&mut self, program: &Program) -> IrProgram {
        let mut functions = Vec::new();

        // First, collect all external function declarations
        for item in &program.items {
            if let SourceItem::FuncDeclaration(decl) = item {
                self.external_functions
                    .insert(decl.signature.name.name.clone());
            }
        }

        // First pass: process all statements to find createThread calls
        // This populates thread_functions before we generate functions
        for item in &program.items {
            if let SourceItem::FuncDefinition(def) = item {
                for stmt in &def.body {
                    self.find_create_thread_calls(stmt);
                }
            }
        }

        eprintln!(
            "DEBUG: thread_functions after first pass: {:?}",
            self.thread_functions
        );

        // Generate scheduler function if there are coroutines
        let mut scheduler_func = None;
        if !self.coroutines.is_empty() {
            scheduler_func = Some(self.generate_scheduler_function());
        }

        for item in &program.items {
            if let SourceItem::FuncDefinition(def) = item {
                // Reset block counter and declared vars for each function
                self.block_counter = 0;
                self.declared_vars.clear();
                let ir_func = self.generate_function(def);
                functions.push(ir_func);
            }
        }

        IrProgram { functions }
    }

    pub fn find_create_thread_calls(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Expression(expr_stmt) => {
                self.find_create_thread_in_expr(&expr_stmt.expr);
            }
            Statement::Block(block_stmt) => {
                for s in &block_stmt.body {
                    self.find_create_thread_calls(s);
                }
            }
            Statement::If(if_stmt) => {
                self.find_create_thread_in_expr(&if_stmt.condition);
                self.find_create_thread_calls(&if_stmt.consequence);
                if let Some(alt) = &if_stmt.alternative {
                    self.find_create_thread_calls(alt);
                }
            }
            Statement::Loop(loop_stmt) => {
                self.find_create_thread_in_expr(&loop_stmt.condition);
                for s in &loop_stmt.body {
                    self.find_create_thread_calls(s);
                }
            }
            Statement::Repeat(repeat_stmt) => {
                self.find_create_thread_in_expr(&repeat_stmt.condition);
                self.find_create_thread_calls(&repeat_stmt.body);
            }
            Statement::Return(_) | Statement::Break(_) => {}
        }
    }

    fn find_create_thread_in_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::CreateThread(ct) => {
                eprintln!("DEBUG: Found createThread for {}", ct.function_name.name);
                self.thread_functions.insert(ct.function_name.name.clone());
                let scheduler = ct
                    .scheduler
                    .as_ref()
                    .map(|s| s.name.clone())
                    .unwrap_or_else(|| "FCFS".to_string());
                self.scheduler_type
                    .insert(ct.function_name.name.clone(), scheduler.clone());

                // Create coroutine info with stack
                self.coroutines.insert(
                    ct.function_name.name.clone(),
                    CoroutineInfo {
                        name: ct.function_name.name.clone(),
                        scheduler,
                        stack_size: 4096, // 4KB stack per coroutine
                    },
                );
                self.current_coroutine_id += 1;
            }
            Expr::Call(call) => {
                self.find_create_thread_in_expr(&call.function);
                for arg in &call.arguments {
                    self.find_create_thread_in_expr(arg);
                }
            }
            Expr::Binary(bin) => {
                self.find_create_thread_in_expr(&bin.left);
                self.find_create_thread_in_expr(&bin.right);
            }
            Expr::Unary(un) => {
                self.find_create_thread_in_expr(&un.operand);
            }
            Expr::Parenthesized(inner) => {
                self.find_create_thread_in_expr(inner);
            }
            Expr::Slice(slice) => {
                self.find_create_thread_in_expr(&slice.array);
            }
            Expr::Identifier(_) | Expr::Literal(_) => {}
        }
    }

    pub fn generate_function(&mut self, def: &FuncDefinition) -> IrFunction {
        let return_type = match &def.signature.return_type {
            Some(ty) => self.convert_type(ty),
            None => IrType::Void,
        };

        let mut parameters = Vec::new();
        let mut locals = HashMap::new();

        if let Some(ref params) = def.signature.parameters {
            for (_i, arg) in params.iter().enumerate() {
                let ty = match &arg.ty {
                    Some(t) => self.convert_type(t),
                    None => IrType::Int,
                };

                parameters.push(IrParameter {
                    name: arg.name.name.clone(),
                    ty: ty.clone(),
                });

                locals.insert(
                    arg.name.name.clone(),
                    IrLocal {
                        name: arg.name.name.clone(),
                        ty,
                        stack_offset: None,
                    },
                );

                // Track declared variables
                self.declared_vars.insert(arg.name.name.clone());
            }
        }

        let mut blocks = Vec::new();
        let mut block_stack: Vec<IrBlock> = Vec::new();

        let entry_id = self.generate_block_id();
        let mut current_block = IrBlock {
            id: entry_id,
            instructions: Vec::new(),
            successors: Vec::new(),
        };

        self.locals = locals.clone();

        for stmt in &def.body {
            self.visit_statement(&mut current_block, &mut block_stack, stmt);
        }

        // After processing all statements:
        // - current_block contains post-loop code
        // - block_stack contains: [header, body, init_jmp, exit] (in that order from visit_loop_statement)
        //
        // We want final order: init_jmp (BB_0), header (BB_1), body (BB_2), post_loop (BB_3)
        // Exit block is empty and gets merged with post-loop code
        //
        // block_stack[0] = header (contains CondBr)
        // block_stack[1] = body (contains body code + Jump to header)
        // block_stack[2] = init_jmp (old current_block with Jump to header)
        // block_stack[3] = exit (empty - merge with post-loop)

        // Add init_jmp block first (BB_0) - it's at index 2 in stack
        if block_stack.len() >= 3 {
            let init_jmp_block = block_stack.remove(2);
            blocks.push(init_jmp_block);
        }

        // Add header block (BB_1)
        if !block_stack.is_empty() {
            let header_block = block_stack.remove(0);
            blocks.push(header_block);
        }

        // Add body block (BB_2)
        if !block_stack.is_empty() {
            let body_block = block_stack.remove(0);
            blocks.push(body_block);
        }

        // Exit block is at index 0 now - just remove it (empty), don't add as separate block
        if !block_stack.is_empty() {
            block_stack.remove(0); // discard empty exit block
        }

        // Add post-loop code with fixed ID (next available is BB_3)
        current_block.id = format!("BB{}", blocks.len());
        blocks.push(current_block);

        let mut func = IrFunction {
            name: def.signature.name.name.clone(),
            return_type,
            parameters,
            blocks,
            locals: self.locals.clone().into_values().collect(),
            used_functions: self.used_functions.clone(),
            is_thread: self.thread_functions.contains(&def.signature.name.name),
        };

        // If this function is marked as a thread (via createThread) AND not external, insert Yield after each instruction
        if self.should_insert_yields(&func.name) {
            for block in &mut func.blocks {
                let mut new_instructions = Vec::new();
                for instr in &block.instructions {
                    new_instructions.push(instr.clone());
                    // Always add a Yield after each instruction
                    new_instructions.push(IrInstruction {
                        opcode: IrOpcode::Yield,
                        result: None,
                        result_type: None,
                        operands: Vec::new(),
                        jump_target: None,
                        true_target: None,
                        false_target: None,
                        span: instr.span,
                    });
                }
                block.instructions = new_instructions;
            }
        }

        func.used_functions = self.used_functions.clone();
        func
    }

    pub fn get_ident_type(&self, id: &Identifier) -> IrType {
        self.locals
            .get(&id.name)
            .map(|l| l.ty.clone())
            .unwrap_or(IrType::Int)
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
            TypeRef::Custom(_) => IrType::Int,
            TypeRef::Array {
                element_type, size, ..
            } => IrType::Array(Box::new(self.convert_type(element_type)), *size as usize),
        }
    }

    fn generate_scheduler_function(&mut self) -> IrFunction {
        let mut blocks = Vec::new();

        // Создаём блоки для scheduler
        let entry_id = self.generate_block_id();
        let mut entry_block = IrBlock {
            id: entry_id,
            instructions: Vec::new(),
            successors: Vec::new(),
        };

        // Scheduler state variables (в виде локальных переменных)
        // current_coroutine - индекс текущей корутины
        // coroutine_count - количество корутин

        // Генерируем loop для round-robin
        let loop_header_id = self.generate_block_id();
        let loop_body_id = self.generate_block_id();
        let loop_end_id = self.generate_block_id();

        // Entry: инициализация
        entry_block.instructions.push(IrInstruction {
            opcode: IrOpcode::Jump,
            result: None,
            result_type: None,
            operands: Vec::new(),
            jump_target: Some(loop_header_id.clone()),
            true_target: None,
            false_target: None,
            span: Span::new(0, 0),
        });
        blocks.push(entry_block);

        // Loop header: выбираем следующую корутину
        let mut header_block = IrBlock {
            id: loop_header_id.clone(),
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        // TODO: логика выбора следующей корутины (FCFS/SPN)
        header_block.instructions.push(IrInstruction {
            opcode: IrOpcode::Jump,
            result: None,
            result_type: None,
            operands: Vec::new(),
            jump_target: Some(loop_body_id.clone()),
            true_target: None,
            false_target: None,
            span: Span::new(0, 0),
        });
        blocks.push(header_block);

        // Loop body: resume корутину
        let mut body_block = IrBlock {
            id: loop_body_id,
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        // CoroutineResume для каждой корутины
        for (i, (name, _info)) in self.coroutines.iter().enumerate() {
            body_block.instructions.push(IrInstruction {
                opcode: IrOpcode::CoroutineResume,
                result: None,
                result_type: None,
                operands: vec![IrOperand::Constant(Constant::Int(i as i64))],
                jump_target: Some(name.clone()),
                true_target: None,
                false_target: None,
                span: Span::new(0, 0),
            });
        }
        // После всех корутин - вернуться к header
        body_block.instructions.push(IrInstruction {
            opcode: IrOpcode::Jump,
            result: None,
            result_type: None,
            operands: Vec::new(),
            jump_target: Some(loop_header_id.clone()),
            true_target: None,
            false_target: None,
            span: Span::new(0, 0),
        });
        blocks.push(body_block);

        IrFunction {
            name: "scheduler".to_string(),
            return_type: IrType::Void,
            parameters: Vec::new(),
            blocks,
            locals: Vec::new(),
            used_functions: Vec::new(),
            is_thread: false,
        }
    }
}

#[allow(dead_code)]
impl Span {
    fn span(&self) -> Span {
        *self
    }
}

impl Statement {
    fn span(&self) -> Span {
        match self {
            Statement::Return(s) => s.span,
            Statement::If(s) => s.span,
            Statement::Loop(s) => s.span,
            Statement::Repeat(s) => s.span,
            Statement::Break(s) => s.span,
            Statement::Expression(s) => s.span,
            Statement::Block(s) => s.span,
        }
    }
}
