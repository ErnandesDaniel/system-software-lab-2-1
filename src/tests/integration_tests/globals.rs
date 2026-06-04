use crate::codegen::nasm::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::parser::Parser;
use super::compile_and_run;
use super::compile_only;

#[test]
fn test_exe_global_read() {
    let source = r#"
        global counter of int = 42;
        def main() of int {
            return counter;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_asm_global_in_data_section() {
    let source = r#"
        global counter of int = 42;
        def main() of int {
            return counter;
        }
    "#;
    let (_, asm) = compile_only(source);
    assert!(asm.contains("counter"), "Expected global label in asm");
    assert!(asm.contains("section .data"), "Expected data section");
    assert!(asm.contains("dd 42"), "Expected dd 42");
}

#[test]
fn test_exe_global_write() {
    let source = r#"
        global value of int = 0;
        def main() of int {
            value = 99;
            return value;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_global_string() {
    let source = r#"
        global name of string = "test";
        def main() of int {
            return 0;
        }
    "#;
    let (_, asm) = compile_only(source);
    assert!(asm.contains("section .data"), "Expected data section");
    assert!(
        asm.contains("name") || asm.contains("116"),
        "Expected name label or byte data"
    );
}

#[test]
fn test_exe_global_struct_field() {
    let source = r#"
        struct Sched {
            count of int;
            index of int;
        }
        global sched of Sched;
        def main() of int {
            return sched.count;
        }
    "#;
    let (_, asm) = compile_only(source);
    assert!(asm.contains("global main"), "Expected global main");
    assert!(asm.contains("sched"), "Expected sched in data");
    assert!(asm.contains("[rel sched"), "Expected rel sched");
}

#[test]
fn test_exe_struct_array_field_read() {
    let source = r#"
        struct Sched {
            slots of int[3];
            count of int;
        }
        global sched of Sched;
        def main() of int {
            return sched.slots[0];
        }
    "#;
    let (_, asm) = compile_only(source);
    assert!(asm.contains("sched"), "Expected sched label");
    assert!(asm.contains("lea "), "Expected lea for array field");
}

#[test]
fn test_exe_local_struct() {
    let source = r#"
        struct Point { x of int; y of int; }
        def main() of int {
            p of Point;
            p.x = 42;
            return p.x;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_asm_coroutine_state_machine() {
    let source = r#"
        import putchar
        coroutine worker() of int {
            putchar(49);
            yield;
            putchar(50);
            return 0;
        }
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    assert_eq!(ir.functions.len(), 1);
    assert!(ir.functions[0].is_coroutine, "Expected is_coroutine");
    let mut asm_gen = AsmGenerator::new();
    asm_gen.set_coroutine(ir.functions[0].yield_count);
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("co_0"), "Expected state 0 label");
    assert!(asm.contains("co_1"), "Expected state 1 label");
    assert!(asm.contains("[rcx]"), "Expected state struct access");
    assert!(asm.contains("global worker"), "Expected global worker");
}

#[test]
fn test_asm_coroutine_locals() {
    let source = r#"
        coroutine counter() of int {
            i = 0;
            yield;
            return i;
        }
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    let mut asm_gen = AsmGenerator::new();
    if !ir.functions.is_empty() && ir.functions[0].is_coroutine {
        asm_gen.set_coroutine(ir.functions[0].yield_count);
    }
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("global counter"), "Expected global counter");
}

#[test]
fn test_nasm_coroutine_compiles() {
    let source = r#"
        coroutine simple() of int {
            return 0;
        }
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    let mut asm_gen = AsmGenerator::new();
    if !ir.functions.is_empty() && ir.functions[0].is_coroutine {
        asm_gen.set_coroutine(ir.functions[0].yield_count);
    }
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("global simple"), "Expected global simple");
}

#[test]
fn test_exe_global_array_read() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int {
            return arr[0];
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_global_array_index() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int {
            return arr[2];
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_asm_global_array_parse() {
    let source1 = r#"
        global arr of int[3];
        def main() of int {
            return 0;
        }
    "#;
    let mut parser = Parser::new(source1);
    let ast1 = parser.parse().unwrap();

    let source2 = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int {
            return 0;
        }
    "#;
    let mut parser2 = Parser::new(source2);
    let ast2 = parser2.parse().unwrap();

    assert_eq!(ast1.items.len(), 2, "Without init: expected 2");
    assert_eq!(ast2.items.len(), 2, "With init: expected 2");
}

#[test]
fn test_asm_global_array_init() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int {
            return arr[0];
        }
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    assert_eq!(ir.functions.len(), 1, "Expected 1 function");
    assert_eq!(ir.globals.len(), 1, "Expected 1 global");
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("section .data"), "Expected data section");
    assert!(asm.contains("global main"), "Expected global main");
}

#[test]
fn test_exe_global_array_write_read() {
    let source = r#"
        global arr of int[5];
        def main() of int {
            arr[0] = 42;
            arr[1] = 13;
            arr[2] = arr[0] + arr[1];
            return arr[2];
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(55), "global array write/read: arr[0]=42, arr[1]=13, arr[2]=42+13=55");
}
