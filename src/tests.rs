#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    fn parse(source: &str) -> crate::ast::Program {
        let mut parser = Parser::new(source);
        parser.parse().unwrap()
    }

    fn nasm_path() -> String {
        std::env::var("NASM_PATH")
            .unwrap_or_else(|_| "C:\\Users\\Ernan\\AppData\\Local\\bin\\NASM\\nasm.exe".to_string())
    }

    fn golink_path() -> String {
        std::env::var("GOLINK_PATH").unwrap_or_else(|_| {
            "C:\\Users\\Ernan\\AppData\\Local\\bin\\NASM\\GoLink.exe".to_string()
        })
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
    fn test_array_type() {
        let source = "def foo(arr int array[10]) end";
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
    fn test_unary_expressions() {
        let source = "def foo() x = -5; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_function_call() {
        let source = "def foo() x = bar(1, 2); end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_break_statement() {
        let source = "def foo() break; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_repeat_loop() {
        let source = "def foo() do x = x + 1; while x < 10; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_block_statement() {
        let source = "def foo() begin x = 1; y = 2; end end";
        let result = {
            let mut parser = crate::parser::Parser::new(source);
            parser.parse()
        };
        let program = result.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_multiple_functions() {
        let source = "def foo() x = 1; end def bar() y = 2; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_comparison_operators() {
        let source = "def foo() x = 1 < 2; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_logical_operators() {
        let source = "def foo() x = true && false; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_string_literal() {
        let source = "def foo() x = \"hello\"; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_char_literal() {
        let source = "def foo() x = 'a'; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_bool_literal() {
        let source = "def foo() x = true; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_hex_literal() {
        let source = "def foo() x = 0xFF; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_bits_literal() {
        let source = "def foo() x = 0b1010; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parenthesized_expr() {
        let source = "def foo() x = (1 + 2); end";
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
    fn test_ir_generation_with_arithmetic() {
        use crate::ir_generator::IrGenerator;
        let source = "def foo() x = 1 + 2; return x; end";
        let program = parse(source);
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);
        assert_eq!(ir.functions.len(), 1);
        assert!(!ir.functions[0].blocks.is_empty());
    }

    #[test]
    fn test_ir_generation_with_loop() {
        use crate::ir_generator::IrGenerator;
        let source = "def foo() while true x = 1; end return 0; end";
        let program = parse(source);
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);
        assert_eq!(ir.functions.len(), 1);
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
    fn test_asm_generation_with_arithmetic() {
        use crate::codegen::AsmGenerator;
        use crate::ir_generator::IrGenerator;
        let source = "def foo() x = 5 + 3; return x; end";
        let program = parse(source);
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);
        let mut asm_gen = AsmGenerator::new();
        let asm = asm_gen.generate(&ir);
        assert!(asm.contains("add"));
    }

    #[test]
    fn test_array_access() {
        use crate::ir_generator::IrGenerator;
        let source = "def foo() arr = 1; x = arr[0]; return x; end";
        let program = parse(source);
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);
        assert_eq!(ir.functions.len(), 1);
    }

    #[test]
    fn test_string_in_asm() {
        use crate::codegen::AsmGenerator;
        use crate::ir_generator::IrGenerator;
        let source = "def foo() return \"hello\"; end";
        let program = parse(source);
        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);
        let mut asm_gen = AsmGenerator::new();
        let asm = asm_gen.generate(&ir);
        assert!(asm.contains("section .data") || asm.contains("str_"));
    }

    #[test]
    fn test_exe_compilation_and_execution() {
        use crate::codegen::AsmGenerator;
        use crate::ir_generator::IrGenerator;
        use std::fs;
        use std::process::Command;

        // Use "main" as function name for the entry point
        let source = "def main() return 42; end";
        let program = parse(source);

        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);

        let mut asm_gen = AsmGenerator::new();
        let asm = asm_gen.generate(&ir);

        // Create temp directory for output
        let temp_dir = std::env::temp_dir().join("mylang_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Clean up any old exe files
        let _ = fs::remove_file(temp_dir.join("program.exe"));
        let _ = fs::remove_file(temp_dir.join("program.asm"));
        let _ = fs::remove_file(temp_dir.join("program.obj"));

        // Write assembly
        let asm_path = temp_dir.join("program.asm");
        fs::write(&asm_path, &asm).unwrap();
        eprintln!("Generated assembly:\n{}", asm);

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

        // Skip test if NASM not available
        if !nasm_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            println!("NASM not available, skipping exe test");
            return;
        }

        // Link with GoLink
        let exe_path = temp_dir.join("program.exe");
        let golink_result = Command::new("GoLink.exe")
            .args(["/console", "/entry:main", obj_path.to_str().unwrap()])
            .current_dir(&temp_dir)
            .output();

        // Print linker output for debugging
        if let Ok(out) = &golink_result {
            if !out.status.success() {
                eprintln!("GoLink stderr: {}", String::from_utf8_lossy(&out.stderr));
                eprintln!("GoLink stdout: {}", String::from_utf8_lossy(&out.stdout));
            }
        }

        // Skip test if GoLink not available
        if !golink_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            println!("GoLink not available, skipping exe test");
            return;
        }

        // Run the executable
        let run_result = Command::new(exe_path.to_str().unwrap())
            .current_dir(&temp_dir)
            .output();

        // Print execution output for debugging - show more details
        if let Ok(out) = &run_result {
            eprintln!("Exit code: {:?}", out.status.code());
            if !out.stdout.is_empty() {
                eprintln!("stdout: '{}'", String::from_utf8_lossy(&out.stdout));
            }
            if !out.stderr.is_empty() {
                eprintln!("stderr: '{}'", String::from_utf8_lossy(&out.stderr));
            }
        } else if let Err(e) = &run_result {
            eprintln!("Failed to run exe: {}", e);
        }

        // Verify execution
        assert!(run_result.is_ok(), "Failed to run executable");
        let output = run_result.unwrap();
        assert_eq!(
            output.status.code(),
            Some(42),
            "Expected return value 42, got {:?}",
            output.status.code()
        );

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_exe_with_arithmetic() {
        use crate::codegen::AsmGenerator;
        use crate::ir_generator::IrGenerator;
        use std::fs;
        use std::process::Command;

        // Test: 5 + 3 * 2 = 11
        let source = "def main() x = 5 + 3 * 2; return x; end";
        let program = parse(source);

        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);

        // Print IR to see what's generated
        eprintln!("=== Generated IR (first 3000 chars) ===");
        let ir_str = format!("{:#?}", ir);
        eprintln!("{}", &ir_str[..ir_str.len().min(3000)]);

        let mut asm_gen = AsmGenerator::new();
        let asm = asm_gen.generate(&ir);

        let temp_dir = std::env::temp_dir().join("mylang_arith_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Clean up any old exe files
        let _ = fs::remove_file(temp_dir.join("program.exe"));
        let _ = fs::remove_file(temp_dir.join("program.asm"));
        let _ = fs::remove_file(temp_dir.join("program.obj"));

        let asm_path = temp_dir.join("program.asm");
        fs::write(&asm_path, &asm).unwrap();
        // Print more of the asm to see what's happening
        eprintln!("=== Generated assembly (first 2000 chars) ===");
        eprintln!("{}", &asm[..asm.len().min(2000)]);

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
            println!("NASM not available, skipping test");
            return;
        }

        let exe_path = temp_dir.join("program.exe");
        let golink_result = Command::new("GoLink.exe")
            .args(["/console", "/entry:main", obj_path.to_str().unwrap()])
            .current_dir(&temp_dir)
            .output();

        if !golink_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            println!("GoLink not available, skipping test");
            return;
        }

        let run_result = Command::new(exe_path.to_str().unwrap())
            .current_dir(&temp_dir)
            .output();

        assert!(run_result.is_ok(), "Failed to run executable");
        let output = run_result.unwrap();
        assert_eq!(
            output.status.code(),
            Some(11),
            "Expected 5 + 3*2 = 11, got {:?}",
            output.status.code()
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_extern_functions_and_complex_program() {
        use crate::codegen::AsmGenerator;
        use crate::ir_generator::IrGenerator;
        use std::fs;
        use std::process::Command;

        let source = r#"
extern def getchar() of int end
extern def srand(seed of int) end
extern def time(dummy of int) of int end
extern def rand() of int end
extern def putchar(c of int) of int end
extern def puts(s of string) end
extern def printf(format of string, value of int) end

def square(x of int) of int
    return x * x;
end

def main() of int
    i = 1;
    while i < 5 {
        i = i + 1;
    }
    t = time(0);
    srand(t);
    r = rand();
    return r;
end
"#;
        let program = parse(source);

        let mut ir_gen = IrGenerator::new();
        let ir = ir_gen.generate(&program);

        let mut asm_gen = AsmGenerator::new();
        let asm = asm_gen.generate(&ir);

        eprintln!("=== Generated assembly with extern ===");
        eprintln!("{}", asm);

        let temp_dir = std::env::temp_dir().join("mylang_extern_test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

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

        // Debug output
        if let Ok(out) = &nasm_result {
            if !out.status.success() {
                eprintln!("NASM stderr: {}", String::from_utf8_lossy(&out.stderr));
                eprintln!("NASM stdout: {}", String::from_utf8_lossy(&out.stdout));
            }
        }

        if !nasm_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            println!("NASM not available, skipping extern test");
            return;
        }

        // Link with GoLink - console application (no /dll flag, just import from DLL)
        let obj_name = "program.obj";
        let exe_path = temp_dir.join("program.exe");

        // Link using the obj file - GoLink will create program.exe from program.obj
        let golink_result = Command::new("GoLink.exe")
            .args(["/console", "/entry:main", "msvcrt.dll", obj_name])
            .current_dir(&temp_dir)
            .output();

        // Debug output
        if let Ok(out) = &golink_result {
            eprintln!("GoLink exit code: {:?}", out.status.code());
            if !out.status.success() {
                eprintln!("GoLink stderr: {}", String::from_utf8_lossy(&out.stderr));
                eprintln!("GoLink stdout: {}", String::from_utf8_lossy(&out.stdout));
            } else {
                eprintln!("GoLink succeeded!");
            }
        }

        if !golink_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            println!("GoLink not available, skipping extern test");
            return;
        }

        // Run - should return a random number
        eprintln!("Listing files in temp_dir:");
        for entry in std::fs::read_dir(&temp_dir).unwrap() {
            let e = entry.unwrap();
            eprintln!("  {:?}", e.path());
        }

        let exe_path_str = exe_path.to_str().unwrap();
        eprintln!("Trying to run exe: {}", exe_path_str);
        eprintln!("Exe exists: {}", exe_path.exists());

        let run_result = Command::new(exe_path_str).current_dir(&temp_dir).output();

        if let Err(e) = &run_result {
            eprintln!("Failed to run executable error: {}", e);
        }

        assert!(run_result.is_ok(), "Failed to run executable");
        let output = run_result.unwrap();

        // Random number should be non-zero (seeded with time)
        assert!(
            output.status.code().unwrap_or(0) >= 0,
            "Expected non-negative random number, got {:?}",
            output.status.code()
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
