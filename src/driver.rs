use std::fs;
use std::path::Path;

use crate::ast;
use crate::codegen::{self, LlvmGenerator};
use crate::CodeGenTarget;
use crate::ir::cfg::CfgMermaidGenerator;
use crate::ir::validator::IrValidator;
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
        if let Err(errors) = IrValidator::validate(&ir_program) {
            for err in errors {
                eprintln!("IR validation error: {}", err);
            }
        }
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

        let has_coroutines = ir.functions.iter().any(|f| f.yield_count > 0);

        for func in &ir.functions {

            let mut gen = codegen::AsmGenerator::new();
            if func.yield_count > 0 {
                gen.set_coroutine(func.yield_count);
            }
            let mut asm = gen.generate_single_function(func);
            if !ir.globals.is_empty() {
                let mut externs = String::new();
                for g in &ir.globals {
                    externs.push_str(&format!("extern {}\n", g.name));
                }
                asm.insert_str(0, &externs);
            }
            let path = Path::new(output_dir).join(format!("{}.asm", func.name));
            
            if let Err(e) = fs::write(&path, &asm) {
                eprintln!("Failed to write assembly: {}", e);
            }
        }

        if !ir.globals.is_empty() {
            let mut globals_asm = String::from("bits 64\ndefault rel\nsection .data\n");
            globals_asm.push_str(&crate::codegen::AsmGenerator::generate_globals_asm(&ir.globals));
            let path = Path::new(output_dir).join("globals.asm");
            let _ = fs::write(&path, &globals_asm);
        }

        let mut obj_files = Vec::new();
        if has_coroutines {
            let mut helper = String::from("bits 64\ndefault rel\n\n");

            // State structs + pointer table
            helper.push_str("section .data\n");
            helper.push_str("co_states dq 0, 0, 0, 0, 0, 0, 0, 0\n");
            for f in ir.functions.iter().filter(|f| f.yield_count > 0) {
                helper.push_str(&format!("state_{} dd 0, 0, 0, 0, 0, 0\n", f.name));
            }

            helper.push_str("\nsection .text\n");
            helper.push_str("global resume_coroutine_nasm\nresume_coroutine_nasm:\n");
            helper.push_str("    ; rcx = index\n");
            helper.push_str("    lea rax, [rel co_states]\n");
            helper.push_str("    mov rax, [rax + rcx * 8]\n");
            helper.push_str("    test rax, rax\n    jz .empty\n");
            helper.push_str("    mov rcx, rax\n");
            helper.push_str("    mov eax, [rcx]\n    cmp eax, -1\n    jne .go\n    mov eax, 1\n    ret\n.go:\n");
            helper.push_str("    push rbp\n    mov rbp, rsp\n    sub rsp, 32\n    call [rcx + 8]\n    mov eax, [rcx + 16]\n    leave\n    ret\n");
            helper.push_str(".empty:\n    mov eax, 1\n    ret\n\n");

            helper.push_str("global create_coroutine_nasm\ncreate_coroutine_nasm:\n");
            helper.push_str("    mov dword [rcx], 0\n    mov [rcx + 8], rdx\n    mov dword [rcx + 16], 0\n    ret\n\n");

            // Init: fill co_states table
            helper.push_str("global coro_init_nasm\n");
            for f in ir.functions.iter().filter(|f| f.yield_count > 0) {
                helper.push_str(&format!("extern {}\n", f.name));
            }
            helper.push_str("coro_init_nasm:\n    push rbp\n    mov rbp, rsp\n");
            let mut idx = 0;
            for f in ir.functions.iter().filter(|f| f.yield_count > 0) {
                helper.push_str(&format!("    lea rcx, [rel state_{}]\n", f.name));
                helper.push_str(&format!("    lea rdx, [rel {}]\n", f.name));
                helper.push_str("    sub rsp, 32\n    call create_coroutine_nasm\n    add rsp, 32\n");
                helper.push_str(&format!("    lea rax, [rel co_states]\n"));
                helper.push_str(&format!("    lea rcx, [rel state_{}]\n", f.name));
                helper.push_str(&format!("    mov [rax + {}], rcx\n", idx * 8));
                idx += 1;
            }
            helper.push_str("    leave\n    ret\n");

            let path = Path::new(output_dir).join("coro_helpers.asm");
            fs::write(&path, helper).ok();

            let obj = Path::new(output_dir).join("coro_helpers.obj");
            let output = Command::new("nasm")
                .args(["-f", "win64", "-o"])
                .arg(obj.to_str().unwrap())
                .arg(path.to_str().unwrap())
                .output();
            if let Ok(out) = output {
                if out.status.success() { obj_files.push(obj); }
            }
        }

        for func in &ir.functions {
            let asm_path = Path::new(output_dir).join(format!("{}.asm", func.name));
            let obj_path = Path::new(output_dir).join(format!("{}.obj", func.name));

            let output = if func.yield_count > 0 {
                Command::new("nasm")
                    .args(["-f", "win64", "-O0", "-o", obj_path.to_str().unwrap(), asm_path.to_str().unwrap()])
                    .output()
            } else {
                Command::new("nasm")
                    .args(["-f", "win64", "-o", obj_path.to_str().unwrap(), asm_path.to_str().unwrap()])
                    .output()
            };

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

        let globals_asm = Path::new(output_dir).join("globals.asm");
        if globals_asm.exists() {
            let globals_obj = Path::new(output_dir).join("globals.obj");
            let output = Command::new("nasm")
                .args(["-f", "win64", "-o"])
                .arg(globals_obj.to_str().unwrap())
                .arg(globals_asm.to_str().unwrap())
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    obj_files.push(globals_obj);
                }
            }
        }

        if !obj_files.is_empty() {
            let exe_path = Path::new(output_dir).join("program.exe");
            let mut args: Vec<String> = obj_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            args.push("-Wl,/subsystem:console".to_string());
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

        // Create lib/ in output dir and copy JNA jar for manual compilation too
        let output_lib = Path::new(output_dir).join("lib");
        let _ = fs::create_dir_all(&output_lib);
        let jna_src = Path::new("src/lib").join("jna-5.14.0.jar");
        let jna_dst = output_lib.join("jna-5.14.0.jar");
        if jna_src.exists() {
            let _ = fs::copy(&jna_src, &jna_dst);
        }

        let jna_cp = format!(".;lib/jna-5.14.0.jar");

        let stub_file = "RuntimeStub.java";
        let stub_output = Command::new("javac")
            .current_dir(output_dir)
            .args(["-cp", &jna_cp, stub_file])
            .output()
            .expect("Failed to run javac");

        if !stub_output.status.success() {
            eprintln!("javac (RuntimeStub.java) failed:");
            eprintln!("{}", String::from_utf8_lossy(&stub_output.stderr));
            std::process::exit(1);
        }

        let runner_file = "MainRunner.java";
        let runner_output = Command::new("javac")
            .current_dir(output_dir)
            .arg(runner_file)
            .output()
            .expect("Failed to run javac");

        if !runner_output.status.success() {
            eprintln!("javac (MainRunner.java) failed:");
            eprintln!("{}", String::from_utf8_lossy(&runner_output.stderr));
            std::process::exit(1);
        }

    }

    fn generate_jvm_stub(&self, output_dir: &str) {
        let stub = r#"import java.io.*;
import java.nio.*;
import java.nio.channels.*;
import java.nio.charset.*;
import java.util.*;
import java.lang.reflect.*;

public class RuntimeStub {
    private static final int SHM_SIZE = 4096;

    private static MappedByteBuffer buf;
    private static HashMap<String, String> store = new HashMap<>();
    private static Random random = new Random();

    private static boolean shmInitialized = false;
    private static Object hEvent;
    private static Object kernel32WaitFn;
    private static Object kernel32SetFn;
    private static Object kernel32ResetFn;

    private static synchronized void ensureShm() {
        if (shmInitialized) return;
        shmInitialized = true;
        try {
            Class<?> fnClass = Class.forName("com.sun.jna.Function");
            java.lang.reflect.Method getFn = fnClass.getMethod("getFunction", String.class, String.class);

            kernel32WaitFn = getFn.invoke(null, "kernel32", "WaitForSingleObject");
            kernel32SetFn = getFn.invoke(null, "kernel32", "SetEvent");
            kernel32ResetFn = getFn.invoke(null, "kernel32", "ResetEvent");

            Object createEventFn = getFn.invoke(null, "kernel32", "CreateEventA");
            java.lang.reflect.Method invokeMethod = fnClass.getMethod("invoke", Class.class, Object[].class);
            hEvent = invokeMethod.invoke(createEventFn, Class.forName("com.sun.jna.Pointer"),
                new Object[]{null, 1, 0, "MyLangSHMEvent"});

            if (hEvent == null) {
                throw new RuntimeException("CreateEventA failed");
            }

            RandomAccessFile file = new RandomAccessFile("mylang_shm.dat", "rw");
            file.setLength(SHM_SIZE);
            FileChannel ch = file.getChannel();
            buf = ch.map(FileChannel.MapMode.READ_WRITE, 0, SHM_SIZE);
            buf.order(ByteOrder.LITTLE_ENDIAN);
            ch.close();

            System.out.println("[RuntimeStub] SHM initialized. PID: " + ProcessHandle.current().pid());
        } catch (ClassNotFoundException e) {
            System.err.println("[RuntimeStub] JNA not found. Add jna.jar to classpath for SHM programs.");
            System.exit(1);
        } catch (Exception e) {
            System.err.println("[RuntimeStub] SHM init error: " + e.getMessage());
            System.exit(1);
        }
    }

    public static void main(String[] args) {
        System.out.println("[RuntimeStub] Starting...");
        int result = Main.call();
        System.exit(result);
    }

    // --- Standard IO ---

    public static int getchar() throws IOException {
        return System.in.read();
    }

    public static int putchar(int c) {
        System.out.print((char) c);
        System.out.flush();
        return c;
    }

    public static int puts(String s) {
        System.out.println(s);
        System.out.flush();
        return s.length();
    }

    public static int printf(String format, int value) {
        System.out.print(format.replace("%d", String.valueOf(value))
                                .replace("%c", String.valueOf((char) value))
                                .replace("%s", String.valueOf(value))
                                .replace("\\n", "\n")
                                .replace("\\t", "\t"));
        System.out.flush();
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
        try { Thread.sleep(ms); } catch (InterruptedException e) {}
    }

    // --- SHM functions (JVM) ---

    public static int shm_read_state_jvm() {
        ensureShm();
        return buf.getInt(0);
    }

    public static int shm_read_byte_jvm(int pos) {
        ensureShm();
        return buf.get(pos) & 0xFF;
    }

    public static String shm_read_str_jvm(int pos) {
        ensureShm();
        int len = 0;
        while (pos + len < SHM_SIZE && buf.get(pos + len) != 0) len++;
        byte[] bytes = new byte[len];
        buf.position(pos);
        buf.get(bytes);
        return new String(bytes, StandardCharsets.UTF_8);
    }

    public static void shm_write_state_jvm(int state) {
        ensureShm();
        buf.putInt(0, state);
    }

    public static void shm_write_resp_jvm(int result, String payload) {
        ensureShm();
        try {
            byte[] pbytes = payload != null ? payload.getBytes(StandardCharsets.UTF_8) : new byte[0];
            int maxLen = SHM_SIZE - 6;
            if (pbytes.length > maxLen) pbytes = java.util.Arrays.copyOf(pbytes, maxLen);
            buf.position(4);
            buf.put((byte) result);
            buf.put(pbytes);
            buf.put((byte) 0);
        } catch (Exception e) {
            System.err.println("[RuntimeStub] Write error: " + e.getMessage());
        }
    }

    public static void shm_wait_event_jvm() {
        ensureShm();
        try {
            java.lang.reflect.Method invokeMethod = kernel32WaitFn.getClass()
                .getMethod("invoke", Class.class, Object[].class);
            invokeMethod.invoke(kernel32WaitFn, int.class, new Object[]{hEvent, 2000});
            invokeMethod.invoke(kernel32ResetFn, int.class, new Object[]{hEvent});
        } catch (Exception e) {
            System.err.println("[RuntimeStub] Event error: " + e.getMessage());
        }
    }

    public static int shm_find_null_jvm(int start) {
        ensureShm();
        for (int i = start; i < SHM_SIZE; i++) {
            if (buf.get(i) == 0) return i;
        }
        return SHM_SIZE;
    }

    // --- Map functions (JVM) ---

    public static int map_put_jvm(String name, String value) {
        synchronized (store) { store.put(name, value); return 1; }
    }

    public static String map_get_jvm(String name) {
        synchronized (store) { return store.get(name); }
    }

    public static int map_has_jvm(String name) {
        synchronized (store) { return store.containsKey(name) ? 1 : 0; }
    }

    public static int map_remove_jvm(String name) {
        synchronized (store) { return store.remove(name) != null ? 1 : 0; }
    }

    public static int map_size_jvm() {
        synchronized (store) { return store.size(); }
    }

    public static String map_key_jvm(int i) {
        synchronized (store) {
            int idx = 0;
            for (String k : store.keySet()) {
                if (idx == i) return k;
                idx++;
            }
            return "";
        }
    }

    public static String map_list_jvm() {
        synchronized (store) { return String.join(",", store.keySet()); }
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
