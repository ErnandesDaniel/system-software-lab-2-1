use std::fs;
use std::path::Path;

use crate::ast;
use crate::codegen::{self, LlvmGenerator};
use crate::CodeGenTarget;
use crate::ir::cfg::CfgMermaidGenerator;
use crate::ir_generator::IrGenerator;
use crate::mermaid::MermaidGenerator;
use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;

pub struct CompilerDriver;

impl CompilerDriver {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(&self, args: &crate::cli::Args) {
        let source = self.read_source(&args.source_path);
        let ast = self.parse(&source);
        self.create_output_dir(&args.output_dir);
        self.generate_ast_diagrams(&ast, &args.output_dir);
        self.run_semantic_analysis(&ast);
        let ir_program = self.generate_ir(&ast);
        self.generate_cfg_diagrams(&ir_program, &args.output_dir);
        
        match args.target {
            CodeGenTarget::NASM => self.generate_nasm(&ir_program, &args.output_dir),
            CodeGenTarget::LLVM => self.generate_llvm(&ir_program, &args.output_dir),
            CodeGenTarget::WASM => self.generate_wasm(&ir_program, &args.output_dir),
            CodeGenTarget::JVM => self.generate_jvm(&ir_program, &args.output_dir),
        }
    }

    fn read_source(&self, path: &str) -> String {
        match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read file: {}", e);
                std::process::exit(1);
            }
        }
    }

    fn parse(&self, source: &str) -> ast::Program {
        let mut parser = Parser::new(source);
        match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                std::process::exit(1);
            }
        }
    }

    fn create_output_dir(&self, path: &str) {
        if let Err(e) = fs::create_dir_all(path) {
            eprintln!("Failed to create output directory: {}", e);
            std::process::exit(1);
        }
    }

    fn generate_ast_diagrams(&self, ast: &ast::Program, output_dir: &str) {
        for item in &ast.items {
            if let ast::SourceItem::FuncDefinition(func) = item {
                let path = Path::new(output_dir)
                    .join(format!("{}-ast.mmd", func.signature.name.name));
                let mut gen = MermaidGenerator::new();
                let diagram = gen.generate_function(func);
                
                if let Err(e) = fs::write(&path, &diagram) {
                    eprintln!("Failed to write AST: {}", e);
                }
            }
        }
    }

    fn run_semantic_analysis(&self, ast: &ast::Program) {
        let mut analyzer = SemanticsAnalyzer::new();
        if let Err(errors) = analyzer.analyze(ast) {
            for err in errors {
                eprintln!("Semantic error: {}", err);
            }
            std::process::exit(1);
        }
    }

    fn generate_ir(&self, ast: &ast::Program) -> crate::ir::IrProgram {
        let mut ir_gen = IrGenerator::new();
        ir_gen.generate(ast)
    }

    fn generate_cfg_diagrams(&self, ir: &crate::ir::IrProgram, output_dir: &str) {
        for func in &ir.functions {
            let path = Path::new(output_dir).join(format!("{}-cfg.mmd", func.name));
            let mut gen = CfgMermaidGenerator::new();
            let diagram = gen.generate_function_only(func);
            
            if let Err(e) = fs::write(&path, &diagram) {
                eprintln!("Failed to write CFG: {}", e);
            }
        }
    }

    fn generate_nasm(&self, ir: &crate::ir::IrProgram, output_dir: &str) {
        use std::process::Command;

        for func in &ir.functions {
            let mut gen = codegen::AsmGenerator::new();
            let asm = gen.generate_single_function(func);
            let path = Path::new(output_dir).join(format!("{}.asm", func.name));
            
            if let Err(e) = fs::write(&path, &asm) {
                eprintln!("Failed to write assembly: {}", e);
            }
        }

        let mut obj_files = Vec::new();
        for func in &ir.functions {
            let asm_path = Path::new(output_dir).join(format!("{}.asm", func.name));
            let obj_path = Path::new(output_dir).join(format!("{}.obj", func.name));

            let output = Command::new("nasm")
                .args(["-f", "win64", "-o"])
                .arg(obj_path.to_str().unwrap())
                .arg(asm_path.to_str().unwrap())
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        obj_files.push(obj_path);
                    } else {
                        eprintln!("NASM failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to run NASM: {}", e),
            }
        }

        if !obj_files.is_empty() {
            let exe_path = Path::new(output_dir).join("program.exe");
            let mut args: Vec<String> = obj_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            args.push("-o".to_string());
            args.push(exe_path.to_string_lossy().to_string());

            match Command::new("clang").args(&args).output() {
                Ok(out) => {
                    if !out.status.success() {
                        eprintln!("Link failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to run Clang: {}", e),
            }
        }
    }

    fn generate_llvm(&self, ir: &crate::ir::IrProgram, output_dir: &str) {
        use std::process::Command;

        let mut gen = LlvmGenerator::new();
        let llvm_ir = gen.generate_program(ir);

        let ll_path = Path::new(output_dir).join("program.ll");
        if let Err(e) = fs::write(&ll_path, &llvm_ir) {
            eprintln!("Failed to write LLVM IR: {}", e);
            return;
        }

        let obj_path = Path::new(output_dir).join("program.obj");
        let compile_result = Command::new("clang")
            .args(["-c", "-o"])
            .arg(obj_path.to_str().unwrap())
            .arg(ll_path.to_str().unwrap())
            .output();

        match compile_result {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!("Clang failed: {}", String::from_utf8_lossy(&out.stderr));
                    return;
                }
            }
            Err(e) => {
                eprintln!("Failed to run Clang: {}", e);
                return;
            }
        }

        let exe_path = Path::new(output_dir).join("program.exe");
        match Command::new("clang")
            .arg(obj_path.to_str().unwrap())
            .arg("-o")
            .arg(exe_path.to_str().unwrap())
            .output()
        {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!("Link failed: {}", String::from_utf8_lossy(&out.stderr));
                }
            }
            Err(e) => eprintln!("Failed to run Clang: {}", e),
        }
    }

    fn generate_wasm(&self, ir: &crate::ir::IrProgram, output_dir: &str) {
        use std::process::Command;

        let mut gen = LlvmGenerator::new();
        let llvm_ir = gen.generate_program(ir);

        let ll_path = Path::new(output_dir).join("program.ll");
        if let Err(e) = fs::write(&ll_path, &llvm_ir) {
            eprintln!("Failed to write LLVM IR: {}", e);
            return;
        }

        let wasm_path = Path::new(output_dir).join("program.wasm");

        let compile_result = Command::new("clang")
            .args([
                "--target=wasm32",
                "-nostdlib",
                "-Wl,--no-entry",
                "-Wl,--export-all",
                "-Wl,--allow-undefined",
                "-o",
                wasm_path.to_str().unwrap(),
                ll_path.to_str().unwrap()
            ])
            .output();

        match compile_result {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!("Clang compile failed: {}", String::from_utf8_lossy(&out.stderr));
                }
            }
            Err(e) => {
                eprintln!("Failed to run Clang: {}", e);
            }
        }
    }

    fn generate_jvm(&self, ir: &crate::ir::IrProgram, output_dir: &str) {
        use crate::codegen::JvmGenerator;
        use std::process::Command;

        let mut gen = JvmGenerator::new();
        let classes = gen.generate_program(ir);

        for (class_name, class_bytes) in classes {
            let path = Path::new(output_dir).join(format!("{}.class", class_name));
            if let Err(e) = fs::write(&path, &class_bytes) {
                eprintln!("Failed to write class file: {}", e);
            }
        }

        self.generate_jvm_stub(output_dir);

        let stub_file = "RuntimeStub.java";
        let _ = Command::new("javac")
            .current_dir(output_dir)
            .args(["-cp", ".", stub_file])
            .output();

        let runner_file = "MainRunner.java";
        let _ = Command::new("javac")
            .current_dir(output_dir)
            .arg(runner_file)
            .output();
    }

    fn generate_jvm_stub(&self, output_dir: &str) {
        let stub = r#"import java.io.*;
import java.util.*;

public class RuntimeStub {
    private static Random random = new Random();

    public static int getchar() throws IOException {
        return System.in.read();
    }

    public static int putchar(int c) {
        System.out.print((char) c);
        return c;
    }

    public static int puts(String s) {
        System.out.println(s);
        return s.length();
    }

    public static int printf(String format, int value) {
        System.out.print(format.replace("%d", String.valueOf(value))
                                    .replace("%c", String.valueOf((char) value))
                                    .replace("%s", String.valueOf(value))
                                    .replace("\\n", "\n")
                                    .replace("\\t", "\t"));
        return value;
    }

    public static int rand() {
        return random.nextInt();
    }

    public static void srand(int seed) {
        random = new Random(seed);
    }

    public static int time(int dummy) {
        return (int)(System.currentTimeMillis() / 1000);
    }

    public static void Sleep(int ms) {
        try {
            Thread.sleep(ms);
        } catch (InterruptedException e) {
        }
    }

    public static void main(String[] args) throws Exception {
        int result = Main.call();
        System.exit(result);
    }
}
"#;
        let stub_path = Path::new(output_dir).join("RuntimeStub.java");
        if let Err(e) = fs::write(&stub_path, stub) {
            eprintln!("Failed to write RuntimeStub: {}", e);
        }

        let runner = r#"import java.lang.reflect.Method;
import java.io.File;
import java.net.URL;
import java.net.URLClassLoader;

public class MainRunner {
    public static void main(String[] args) throws Exception {
        if (args.length < 1) {
            printUsage();
            System.exit(1);
        }

        String func = args[0];
        int[] params = new int[args.length - 1];
        for (int i = 1; i < args.length; i++) {
            params[i - 1] = Integer.parseInt(args[i]);
        }

        File currentDir = new File(".");
        URL[] urls = new URL[] { currentDir.toURI().toURL() };
        URLClassLoader classLoader = new URLClassLoader(urls, MainRunner.class.getClassLoader());

        String className = capitalizeFirst(func);
        Class<?> clazz;
        
        try {
            clazz = classLoader.loadClass(className);
        } catch (ClassNotFoundException e) {
            if (func.equals("main")) {
                clazz = classLoader.loadClass("Main");
            } else {
                System.err.println("Unknown function: " + func);
                System.err.println("Class not found: " + className);
                System.exit(1);
                return;
            }
        }

        Method targetMethod = null;
        for (Method method : clazz.getMethods()) {
            if (method.getName().equals("call") && method.getParameterCount() == params.length) {
                targetMethod = method;
                break;
            }
        }

        if (targetMethod == null) {
            System.err.println("Method 'call" + params.length + "' not found in class " + className);
            System.exit(1);
            return;
        }

        Object result = targetMethod.invoke(null, (Object[]) boxParams(params));
        System.out.println(result);
    }

    private static void printUsage() {
        System.out.println("Usage: java MainRunner <function> [args...]");
        System.out.println();
        System.out.println("Available functions:");
        System.out.println("  main             - main function");
        System.out.println("  factorial n      - n factorial");
        System.out.println("  power base exp   - base^exp");
        System.out.println("  sum a b          - a + b");
        System.out.println("  diff a b         - a - b");
        System.out.println("  product a b      - a * b");
    }

    private static String capitalizeFirst(String s) {
        if (s.isEmpty()) return s;
        return Character.toUpperCase(s.charAt(0)) + s.substring(1);
    }

    private static Integer[] boxParams(int[] params) {
        Integer[] result = new Integer[params.length];
        for (int i = 0; i < params.length; i++) {
            result[i] = params[i];
        }
        return result;
    }
}
"#;
        let runner_path = Path::new(output_dir).join("MainRunner.java");
        if let Err(e) = fs::write(&runner_path, runner) {
            eprintln!("Failed to write MainRunner: {}", e);
        }
    }
}
