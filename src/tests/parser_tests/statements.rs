use crate::tests::parse;

fn as_func<'a>(program: &'a crate::ast::Program, idx: usize) -> &'a crate::ast::FuncDefinition {
    if let crate::ast::SourceItem::FuncDefinition(f) = &program.items[idx] {
        f
    } else {
        panic!("item {idx} is not FuncDefinition");
    }
}

#[test]
fn test_if_statement() {
    let program = parse("def foo() { if (x) { return 1; } }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_while_loop() {
    let program = parse("def foo() { x = 1; while (x < 10) { x = x + 1; } }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_begin_end_block() {
    let program = parse("def foo() { { x = 1; } }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_nested_while_loops() {
    let source = r#"
        def foo() {
            i = 0;
            while (i < 3) {
                j = 0;
                while (j < 2) {
                    j = j + 1;
                }
                i = i + 1;
            }
        }
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_if_with_else() {
    let source = r#"
        def foo(x of int) of int {
            if (x > 0) {
                return 1;
            } else {
                return 0;
            }
        }
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_multiple_statements_in_sequence() {
    let source = r#"
        def main() of int {
            a = 1;
            b = 2;
            c = a + b;
            return c;
        }
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_stmt_return_int() {
    let program = parse("def f() of int { return 42; }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::Return(_)));
}

#[test]
fn test_stmt_return_void() {
    let program = parse("def f() { return; }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::Return(_)));
}

#[test]
fn test_stmt_return_expr() {
    let program = parse("def f() of int { return x + 1; }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::Return(_)));
}

#[test]
fn test_stmt_if_simple() {
    let program = parse("def f() { if (x) { return 1; } }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::If(_)));
}

#[test]
fn test_stmt_if_else() {
    let program = parse("def f() { if (x) { a; } else { b; } }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::If(_)));
}

#[test]
fn test_stmt_if_else_if() {
    let program = parse("def f() { if (x) { a; } else if (y) { b; } else { c; } }");
    let f = as_func(&program, 0);
    if let crate::ast::Statement::If(s) = &f.body[0] {
        assert_eq!(s.else_ifs.len(), 1);
        assert!(s.else_body.is_some());
    } else {
        panic!("expected If");
    }
}

#[test]
fn test_stmt_if_multi_else_if() {
    let program = parse("def f() { if (a) { x; } else if (b) { y; } else if (c) { z; } }");
    let f = as_func(&program, 0);
    if let crate::ast::Statement::If(s) = &f.body[0] {
        assert_eq!(s.else_ifs.len(), 2);
    }
}

#[test]
fn test_stmt_while() {
    let program = parse("def f() { while (x < 10) { x = x + 1; } }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::Loop(_)));
}

#[test]
fn test_stmt_until() {
    let program = parse("def f() { until (done) { } }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::Loop(_)));
}

#[test]
fn test_stmt_repeat_while() {
    let program = parse("def f() { { x = x + 1; } while (x < 5); }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::Repeat(_)));
}

#[test]
fn test_stmt_repeat_until() {
    let program = parse("def f() { { x = x + 1; } until (x == 5); }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::Repeat(_)));
}

#[test]
fn test_stmt_break() {
    let program = parse("def f() { while (true) { break; } }");
    let f = as_func(&program, 0);
    if let crate::ast::Statement::Loop(l) = &f.body[0] {
        assert!(matches!(l.body[0], crate::ast::Statement::Break(_)));
    }
}

#[test]
fn test_stmt_block() {
    let program = parse("def f() { { a = 1; b = 2; } }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::Block(_)));
}

#[test]
fn test_stmt_var_decl() {
    let program = parse("def f() { x of int; }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::VarDecl(_)));
}

#[test]
fn test_stmt_var_decl_with_array() {
    let program = parse("def f() { arr of int[10]; }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::VarDecl(_)));
}

#[test]
fn test_stmt_yield() {
    let program = parse("coroutine f() { yield; }");
    if let crate::ast::SourceItem::CoroutineDef(c) = &program.items[0] {
        assert!(matches!(c.body[0], crate::ast::Statement::Yield(_)));
    }
}

#[test]
fn test_stmt_expression_only() {
    let program = parse("def f() { 42; }");
    let f = as_func(&program, 0);
    assert!(matches!(f.body[0], crate::ast::Statement::Expression(_)));
}

#[test]
fn test_stmt_nested_if_in_loop() {
    let program = parse("def f() { while (true) { if (x) { break; } } }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_stmt_nested_loop_in_if() {
    let program = parse("def f() { if (x) { while (y) { z; } } }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_stmt_multiple_in_func() {
    let program = parse("def f() { a; b; c; }");
    let f = as_func(&program, 0);
    assert_eq!(f.body.len(), 3);
}

#[test]
fn test_stmt_empty_body() {
    let program = parse("def f() { }");
    let f = as_func(&program, 0);
    assert_eq!(f.body.len(), 0);
}

#[test]
fn test_stmt_var_decl_then_assign() {
    let program = parse("def f() { x of int; x = 5; }");
    let f = as_func(&program, 0);
    assert_eq!(f.body.len(), 2);
}
