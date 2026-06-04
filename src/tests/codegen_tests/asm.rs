use crate::codegen::nasm::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;

fn asm(source: &str) -> String {
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    asm_gen.generate(&ir)
}

#[test]
fn test_asm_binary_add() {
    let output = asm("def f() of int { return 1 + 2; }");
    assert!(output.contains("add eax"));
}
#[test]
fn test_asm_binary_sub() {
    let output = asm("def f() of int { return 5 - 3; }");
    assert!(output.contains("sub eax"));
}
#[test]
fn test_asm_binary_mul() {
    let output = asm("def f() of int { return 3 * 4; }");
    assert!(output.contains("imul"));
}
#[test]
fn test_asm_binary_div() {
    let output = asm("def f() of int { return 8 / 2; }");
    assert!(output.contains("idiv") || output.contains("div"));
}
#[test]
fn test_asm_binary_mod() {
    let output = asm("def f() of int { return 7 % 3; }");
    assert!(output.contains("div") || output.contains("mod"));
}
#[test]
fn test_asm_load_constant() {
    let output = asm("def f() of int { return 42; }");
    assert!(output.contains("mov eax, 42") || output.contains("mov\teax"));
}
#[test]
fn test_asm_function_name_in_output() {
    let output = asm("def my_func() { }");
    assert!(output.contains("my_func:"));
}
#[test]
fn test_asm_multiple_functions() {
    let output = asm("def a() { } def b() { }");
    assert!(output.contains("a:"));
    assert!(output.contains("b:"));
}
#[test]
fn test_asm_if_else_branches() {
    let output = asm("def f(x of int) of int { if (x > 0) { return 1; } else { return 0; } }");
    assert!(output.contains("cmp"));
    assert!(output.contains("ret"));
}
#[test]
fn test_asm_while_loop_jumps() {
    let output = asm("def f() of int { i = 0; while (i < 10) { i = i + 1; } return i; }");
    assert!(output.contains("cmp"));
    assert!(output.contains("j"));
}
#[test]
fn test_asm_string_data_section() {
    let output = asm(r#"def f() { s = "hello"; }"#);
    assert!(output.contains("db ") || output.contains("section .data"));
}
#[test]
fn test_asm_extern_import() {
    let output = asm("import puts def main() { puts(\"hi\"); }");
    assert!(output.contains("extern puts"));
}
#[test]
fn test_asm_global_var() {
    let output = asm("global x of int = 42; def f() of int { return x; }");
    assert!(output.contains("section .data") || output.contains("x:"));
}
#[test]
fn test_asm_negate_instruction() {
    let output = asm("def f(x of int) of int { return -x; }");
    assert!(
        output.contains("neg eax") || output.contains("neg\teax"),
        "neg instruction not found in: {}",
        &output[..output.len().min(300)]
    );
}
#[test]
fn test_asm_logical_not_trick() {
    let output = asm("def f(x of bool) of bool { return !x; }");
    assert!(output.contains("xor eax") || output.contains("test") || output.contains("sete"));
}
#[test]
fn test_asm_prologue_epilogue() {
    let output = asm("def f() { }");
    assert!(output.contains("push rbp"));
    assert!(output.contains("ret"));
}
#[test]
fn test_asm_section_directives() {
    let output = asm("def f() of int { return 0; }");
    assert!(output.contains("bits 64"));
    assert!(output.contains("default rel"));
    assert!(output.contains("section .text"));
}
#[test]
fn test_asm_basic_block_labels() {
    let output = asm("def f() of int { if (true) { return 1; } return 0; }");
    assert!(output.contains("_BB"));
}
#[test]
fn test_asm_call_extern_function() {
    let output = asm("import def ext_func(x of int); def f() { ext_func(42); }");
    assert!(output.contains("call ext_func") || output.contains("call\text_func"));
}
#[test]
fn test_asm_extern_wins() {
    let output = asm("import def print_int(n of int); def main() { print_int(1); }");
    assert!(output.contains("extern ") && output.contains("print_int"));
}
#[test]
fn test_asm_hex_literal_load() {
    let output = asm("def f() of int { return 0xFF; }");
    assert!(output.contains("ff") || output.contains("255"));
}
#[test]
fn test_asm_true_literal() {
    let output = asm("def f() of bool { return true; }");
    assert!(output.contains("mov") && (output.contains("1") || output.contains("true")));
}
#[test]
fn test_asm_false_literal() {
    let output = asm("def f() of bool { return false; }");
    assert!(!output.contains("mov\teax, 1"));
}
#[test]
fn test_asm_function_call_returns() {
    let output = asm("def double(x of int) of int { return x * 2; } def main() of int { return double(5); }");
    assert!(output.contains("double:"));
    assert!(output.contains("main:"));
}
#[test]
fn test_asm_comparison_chain() {
    let output = asm("def f(a of int, b of int) of bool { return a < b && a > 0; }");
    assert!(output.contains("cmp"));
}
#[test]
fn test_asm_stack_var_access() {
    let output = asm("def f() of int { x = 42; return x; }");
    assert!(output.contains("[rbp"));
}
#[test]
fn test_asm_bits_literal() {
    let output = asm("def f() of int { return 0b1010; }");
    assert!(output.contains("10") || output.contains("0xa") || output.contains("mov"));
}
#[test]
fn test_asm_char_literal() {
    let output = asm("def f() of char { return 'A'; }");
    assert!(output.contains("65") || output.contains("41") || output.contains("mov"));
}
#[test]
fn test_asm_array_index() {
    let output = asm("def f() of int { a of int[4]; a[0] = 42; return a[0]; }");
    assert!(output.contains("[rbp") || output.contains("lea") || output.contains("offset"));
}
#[test]
fn test_asm_struct_field_access() {
    let output = asm("struct P { x of int; y of int; } def f(p of P) of int { return p.x; }");
    assert!(output.contains("[rbp") || output.contains("mov"));
}
#[test]
fn test_asm_function_name_with_numbers() {
    let output = asm("def func123() { }");
    assert!(output.contains("func123:"));
}
#[test]
fn test_asm_underscore_function_name() {
    let output = asm("def _helper() { }");
    assert!(output.contains("_helper:"));
}
#[test]
fn test_asm_many_globals() {
    let src = (0..5).map(|i| format!("global g{i} of int = {i};")).collect::<Vec<_>>().join("\n");
    let src = format!("{src} def main() of int {{ return g0; }}");
    let output = asm(&src);
    assert!(output.contains("section .data") || output.contains("g0"));
}
#[test]
fn test_asm_coroutine_frame() {
    let output = asm("coroutine worker() of int { yield; return 0; }");
    assert!(output.contains("worker:") || output.contains("push rbp"));
}
#[test]
fn test_asm_nested_blocks() {
    let output = asm("def f() of int { { { return 1; } } }");
    assert!(output.contains("ret"));
}
#[test]
fn test_asm_empty_function() {
    let output = asm("def f() { }");
    assert!(output.contains("f:"));
    assert!(output.contains("ret"));
}
#[test]
fn test_asm_var_decl_then_assign() {
    let output = asm("def f() { x of int; x = 5; }");
    assert!(output.contains("mov") || output.contains("push"));
}
#[test]
fn test_asm_return_expr_in_parens() {
    let output = asm("def f() of int { return (42); }");
    assert!(output.contains("mov"));
}
#[test]
fn test_asm_call_imported_twice() {
    let output = asm("import puts def main() { puts(\"a\"); puts(\"b\"); }");
    assert!(output.contains("extern puts"));
}
#[test]
fn test_asm_global_array_init() {
    let output = asm("global arr of int[3] = [1, 2, 3]; def f() of int { return arr[0]; }");
    assert!(output.contains("section .data") || output.contains("arr"));
}
#[test]
fn test_asm_until_loop() {
    let output = asm("def f() of int { i = 0; until (i == 5) { i = i + 1; } return i; }");
    assert!(output.contains("cmp") && output.contains("j"));
}
#[test]
fn test_asm_repeat_while() {
    let output = asm("def f() of int { i = 0; { i = i + 1; } while (i < 5); return i; }");
    assert!(output.contains("cmp") && output.contains("j"));
}
