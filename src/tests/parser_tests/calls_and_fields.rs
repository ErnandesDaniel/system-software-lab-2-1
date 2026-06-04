use crate::tests::parse;

fn first_expr(source: &str) -> crate::ast::Expr {
    let program = parse(source);
    let stmt = &program.items[0];
    if let crate::ast::SourceItem::FuncDefinition(f) = stmt {
        if let crate::ast::Statement::Expression(es) = &f.body[0] {
            return es.expr.clone();
        }
    }
    panic!("not an expression statement");
}

#[test]
fn test_expr_call_no_args() {
    let e = first_expr("def f() { foo(); }");
    assert!(matches!(e, crate::ast::Expr::Call(_)));
}

#[test]
fn test_expr_call_one_arg() {
    let e = first_expr("def f() { foo(1); }");
    if let crate::ast::Expr::Call(c) = e {
        assert_eq!(c.arguments.len(), 1);
    }
}

#[test]
fn test_expr_call_two_args() {
    let e = first_expr("def f() { add(1, 2); }");
    if let crate::ast::Expr::Call(c) = e {
        assert_eq!(c.arguments.len(), 2);
    }
}

#[test]
fn test_expr_call_three_args() {
    let e = first_expr("def f() { foo(1, 2, 3); }");
    if let crate::ast::Expr::Call(c) = e {
        assert_eq!(c.arguments.len(), 3);
    }
}

#[test]
fn test_expr_call_nested() {
    let e = first_expr("def f() { foo(bar()); }");
    assert!(matches!(e, crate::ast::Expr::Call(_)));
}

#[test]
fn test_expr_call_complex_arg() {
    let e = first_expr("def f() { foo(1 + 2); }");
    assert!(matches!(e, crate::ast::Expr::Call(_)));
}

#[test]
fn test_expr_field_access_simple() {
    let e = first_expr("def f() { p.x; }");
    assert!(matches!(e, crate::ast::Expr::FieldAccess(_, _, _)));
}

#[test]
fn test_expr_field_access_chained() {
    let e = first_expr("def f() { a.b.c; }");
    if let crate::ast::Expr::FieldAccess(inner, id, _) = e {
        assert_eq!(id.name, "c");
        assert!(matches!(*inner, crate::ast::Expr::FieldAccess(_, _, _)));
    }
}

#[test]
fn test_expr_field_access_on_call() {
    let e = first_expr("def f() { get_point().x; }");
    assert!(matches!(e, crate::ast::Expr::FieldAccess(_, _, _)));
}

#[test]
fn test_expr_array_index() {
    let e = first_expr("def f() { arr[0]; }");
    assert!(matches!(e, crate::ast::Expr::Slice(_)));
}

#[test]
fn test_expr_array_index_var() {
    let e = first_expr("def f() { arr[i]; }");
    assert!(matches!(e, crate::ast::Expr::Slice(_)));
}

#[test]
fn test_expr_array_index_expr() {
    let e = first_expr("def f() { arr[i + 1]; }");
    assert!(matches!(e, crate::ast::Expr::Slice(_)));
}

#[test]
fn test_expr_array_index_field_access() {
    let e = first_expr("def f() { arr[p.x]; }");
    assert!(matches!(e, crate::ast::Expr::Slice(_)));
}

#[test]
fn test_expr_array_literal_empty() {
    let e = first_expr("def f() { []; }");
    assert!(matches!(e, crate::ast::Expr::ArrayLiteral(..)));
}

#[test]
fn test_expr_array_literal_one() {
    let e = first_expr("def f() { [1]; }");
    if let crate::ast::Expr::ArrayLiteral(vals, _) = e {
        assert_eq!(vals.len(), 1);
    }
}

#[test]
fn test_expr_array_literal_multi() {
    let e = first_expr("def f() { [1, 2, 3]; }");
    if let crate::ast::Expr::ArrayLiteral(vals, _) = e {
        assert_eq!(vals.len(), 3);
    }
}

#[test]
fn test_expr_array_literal_nested() {
    let e = first_expr("def f() { [[1, 2], [3, 4]]; }");
    assert!(matches!(e, crate::ast::Expr::ArrayLiteral(..)));
}

#[test]
fn test_expr_assign_simple() {
    let e = first_expr("def f() { x = 5; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
    if let crate::ast::Expr::Binary(b) = e {
        assert!(matches!(b.operator, crate::ast::BinaryOp::Assign));
    }
}

#[test]
fn test_expr_assign_expression() {
    let e = first_expr("def f() { x = y + 1; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_assign_array_index() {
    let e = first_expr("def f() { arr[0] = 42; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_assign_field() {
    let e = first_expr("def f() { p.x = 5; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_chained_assign() {
    let e = first_expr("def f() { x = y = 5; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_chained_field_access_and_call() {
    let e = first_expr("def f() { a.b.c(); }");
    assert!(matches!(e, crate::ast::Expr::Call(_)));
}

#[test]
fn test_expr_chained_call_on_call() {
    let e = first_expr("def f() { foo()(); }");
    assert!(matches!(e, crate::ast::Expr::Call(_)));
}

#[test]
fn test_expr_chained_array_index_and_field() {
    let e = first_expr("def f() { arr[0].field; }");
    assert!(matches!(e, crate::ast::Expr::FieldAccess(_, _, _)));
}

#[test]
fn test_expr_complex_arithmetic() {
    let e = first_expr("def f() { a + b * c - d / e % f; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_mixed_logical_and_comparison() {
    let e = first_expr("def f() { a < b && c > d || e == f; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_nested_calls_with_array() {
    let e = first_expr("def f() { foo(bar(arr[0])); }");
    assert!(matches!(e, crate::ast::Expr::Call(_)));
}
