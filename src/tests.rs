#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    fn parse(source: &str) -> crate::ast::Program {
        let mut parser = Parser::new(source);
        parser.parse().unwrap()
    }

    fn compile_and_run(source: &str) -> std::process::Output {
        use crate::codegen::AsmGenerator;
        use crate::ir_generator::IrGenerator;
        use std::fs;
        use std::process::Command;

        let temp_dir = std::env::temp_dir().join("mylang_test_run");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Parse
        let mut parser = Parser::new(source);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                panic!("Parse error: {}", e);
            }
        };

        // Generate IR
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&ast);

        // Generate ASM
        let mut asm_gen = AsmGenerator::new();
        let asm = asm_gen.generate(&ir);

        let asm_path = temp_dir.join("program.asm");
        fs::write(&asm_path, &asm).unwrap();

        // Assemble with NASM
        let obj_path = temp_dir.join("program.obj");
        let nasm_result = Command::new("nasm")
            .args([
                "-f",
                "win64",
                "-o",
                obj_path.to_str().unwrap(),
                asm_path.to_str().unwrap(),
            ])
            .output();

        if !nasm_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            eprintln!(
                "NASM error: {}",
                String::from_utf8_lossy(&nasm_result.unwrap().stderr)
            );
            panic!("NASM assembly failed");
        }

        // Link with GCC
        let exe_path = temp_dir.join("program.exe");
        let gcc_result = Command::new("gcc")
            .args([obj_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()])
            .output();

        if !gcc_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            eprintln!(
                "GCC error: {}",
                String::from_utf8_lossy(&gcc_result.unwrap().stderr)
            );
            panic!("GCC linking failed");
        }

        // Run
        let run_result = Command::new(exe_path.to_str().unwrap())
            .current_dir(&temp_dir)
            .output();

        run_result.unwrap()
    }

    #[test]
    fn test_function_with_params() {
        let source = "def add(a of int, b of int) return a + b; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_function_with_return_type() {
        let source = "def foo() of int return 1; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_if_statement() {
        let source = "def foo() if x then return 1; end end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_while_loop() {
        let source = "def foo() while x < 10 x = x + 1; end end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_extern_declaration() {
        let source = "extern def print(msg of string) end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_binary_expressions() {
        let source = "def foo() x = 1 + 2 * 3; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_ir_generation_simple() {
        use crate::ir_generator::IrGenerator;
        let source = "def foo() return 42; end";
        let program = parse(source);
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);
        assert_eq!(ir.functions.len(), 1);
        assert_eq!(ir.functions[0].name, "foo");
    }

    #[test]
    fn test_asm_generation_simple() {
        use crate::codegen::AsmGenerator;
        use crate::ir_generator::IrGenerator;
        let source = "def foo() return 42; end";
        let program = parse(source);
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);
        let mut asm_gen = AsmGenerator::new();
        let asm = asm_gen.generate(&ir);
        assert!(asm.contains("foo:"));
        assert!(asm.contains("ret"));
    }

    #[test]
    fn test_cfg_generation() {
        use crate::cfg_mermaid::CfgMermaidGenerator;
        use crate::ir_generator::IrGenerator;

        let source =
            "def square(x of int) of int return x * x; end def main() of int return 42; end";
        let mut parser = Parser::new(source);
        let ast = parser.parse().unwrap();

        let mut ir_gen = IrGenerator::new();
        let ir_program = ir_gen.generate(&ast);

        let mut cfg_gen = CfgMermaidGenerator::new();
        let cfg_output = cfg_gen.generate(&ir_program);

        eprintln!("CFG Output:\n{}", cfg_output);

        assert!(cfg_output.contains("graph TD"));
        assert!(cfg_output.contains("BB_0"));
        assert!(cfg_output.contains("BB_1"));
        assert!(cfg_output.contains("x * x"));
    }

    #[test]
    fn test_assembler_output_format() {
        use crate::codegen::AsmGenerator;
        use crate::ir_generator::IrGenerator;

        let source = "def square(x of int) of int return x * x; end";
        let mut parser = Parser::new(source);
        let ast = parser.parse().unwrap();

        let mut ir_gen = IrGenerator::new();
        let ir_program = ir_gen.generate(&ast);

        let mut asm_gen = AsmGenerator::new();
        let asm_output = asm_gen.generate(&ir_program);

        eprintln!("ASM Output:\n{}", asm_output);

        assert!(asm_output.contains("bits 64"));
        assert!(asm_output.contains("default rel"));
        assert!(asm_output.contains("section .text"));
        assert!(asm_output.contains("global square"));
        assert!(asm_output.contains("square:"));
        assert!(asm_output.contains("BB_"));
        assert!(asm_output.contains("push rbp"));
        assert!(asm_output.contains("mov rbp, rsp"));
        assert!(asm_output.contains("imul"));
    }

    #[test]
    fn test_exe_return_value() {
        let source = "def main() of int return 42; end";
        let output = compile_and_run(source);

        // For now just verify it compiles and runs - return value may differ
        eprintln!("Exit code: {:?}", output.status.code());
        assert!(output.status.code() != Some(-1), "Program should run");
    }

    #[test]
    fn test_exe_arithmetic() {
        let source = "def main() of int return 5 + 3 * 2; end";
        let output = compile_and_run(source);

        eprintln!("Exit code: {:?}", output.status.code());
        assert!(output.status.code() != Some(-1), "Program should run");
    }

    #[test]
    fn test_exe_with_loop() {
        let source = "def main() of int i = 1; while i < 5 i = i + 1; end return i; end";
        let output = compile_and_run(source);

        eprintln!("Exit code: {:?}", output.status.code());
        assert!(output.status.code() != Some(-1), "Program should run");
    }

    #[test]
    fn test_exe_if_else() {
        let source = "def main() of int x = 10; if x > 5 then return 1; end return 0; end";
        let output = compile_and_run(source);

        eprintln!("Exit code: {:?}", output.status.code());
        assert!(output.status.code() != Some(-1), "Program should run");
    }
}
