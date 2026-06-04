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
fn test_binary_expressions() {
    let program = parse("def foo() { x = 1 + 2 * 3; }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_string_literal_assignment() {
    let program = parse("def foo() { s = \"hello\"; }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_array_indexed_assignment() {
    let program = parse("def foo() { arr[0] = 1; }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_expr_binary_add() {
    let e = first_expr("def f() { 1 + 2; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
    if let crate::ast::Expr::Binary(b) = e {
        assert!(matches!(b.operator, crate::ast::BinaryOp::Add));
    }
}

#[test]
fn test_expr_binary_mul() {
    let e = first_expr("def f() { 3 * 4; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_sub() {
    let e = first_expr("def f() { 10 - 3; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_div() {
    let e = first_expr("def f() { 8 / 2; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_mod() {
    let e = first_expr("def f() { 7 % 3; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_eq() {
    let e = first_expr("def f() { 1 == 2; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_ne() {
    let e = first_expr("def f() { 1 != 2; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_lt() {
    let e = first_expr("def f() { 1 < 2; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_gt() {
    let e = first_expr("def f() { 2 > 1; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_le() {
    let e = first_expr("def f() { 1 <= 2; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_ge() {
    let e = first_expr("def f() { 2 >= 1; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_and() {
    let e = first_expr("def f() { true && false; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_or() {
    let e = first_expr("def f() { true || false; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_bitand() {
    let e = first_expr("def f() { 1 & 3; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_bitor() {
    let e = first_expr("def f() { 1 | 2; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_binary_bitxor() {
    let e = first_expr("def f() { 1 ^ 3; }");
    assert!(matches!(e, crate::ast::Expr::Binary(_)));
}

#[test]
fn test_expr_precedence_mul_before_add() {
    let e = first_expr("def f() { 1 + 2 * 3; }");
    if let crate::ast::Expr::Binary(b) = e {
        assert!(matches!(b.operator, crate::ast::BinaryOp::Add));
        if let crate::ast::Expr::Binary(r) = &*b.right {
            assert!(matches!(r.operator, crate::ast::BinaryOp::Multiply));
        } else {
            panic!("expected multiply as right child");
        }
    }
}

#[test]
fn test_expr_precedence_add_before_compare() {
    let e = first_expr("def f() { 1 + 2 < 3; }");
    if let crate::ast::Expr::Binary(b) = e {
        assert!(matches!(b.operator, crate::ast::BinaryOp::Less));
    }
}

#[test]
fn test_expr_precedence_compare_before_and() {
    let e = first_expr("def f() { 1 < 2 && 3 > 4; }");
    if let crate::ast::Expr::Binary(b) = e {
        assert!(matches!(b.operator, crate::ast::BinaryOp::And));
    }
}

#[test]
fn test_expr_unary_negate() {
    let e = first_expr("def f() { -x; }");
    assert!(matches!(e, crate::ast::Expr::Unary(_)));
    if let crate::ast::Expr::Unary(u) = e {
        assert!(matches!(u.operator, crate::ast::UnaryOp::Negate));
    }
}

#[test]
fn test_expr_unary_not() {
    let e = first_expr("def f() { !x; }");
    assert!(matches!(e, crate::ast::Expr::Unary(_)));
    if let crate::ast::Expr::Unary(u) = e {
        assert!(matches!(u.operator, crate::ast::UnaryOp::Not));
    }
}

#[test]
fn test_expr_unary_bitnot() {
    let e = first_expr("def f() { ~x; }");
    assert!(matches!(e, crate::ast::Expr::Unary(_)));
    if let crate::ast::Expr::Unary(u) = e {
        assert!(matches!(u.operator, crate::ast::UnaryOp::BitNot));
    }
}

#[test]
fn test_expr_unary_negate_dot_precedence() {
    let e = first_expr("def f() { -a.b; }");
    if let crate::ast::Expr::Unary(u) = e {
        assert!(matches!(u.operator, crate::ast::UnaryOp::Negate));
        if let crate::ast::Expr::FieldAccess(_, id, _) = &*u.operand {
            assert_eq!(id.name, "b");
        } else {
            panic!("expected field access after unary");
        }
    }
}

#[test]
fn test_expr_parenthesized() {
    let e = first_expr("def f() { (42); }");
    assert!(matches!(e, crate::ast::Expr::Parenthesized(_)));
}

#[test]
fn test_expr_parenthesized_arithmetic() {
    let e = first_expr("def f() { (1 + 2) * 3; }");
    if let crate::ast::Expr::Binary(b) = e {
        assert!(matches!(b.operator, crate::ast::BinaryOp::Multiply));
    }
}

#[test]
fn test_expr_nested_parentheses() {
    let e = first_expr("def f() { ((1 + 2)); }");
    assert!(matches!(e, crate::ast::Expr::Parenthesized(_)));
}

#[test]
fn test_expr_literal_dec() {
    let e = first_expr("def f() { 42; }");
    if let crate::ast::Expr::Literal(lit, _) = e {
        assert!(matches!(lit, crate::ast::Literal::Dec(42)));
    }
}

#[test]
fn test_expr_literal_hex() {
    let e = first_expr("def f() { 0xFF; }");
    if let crate::ast::Expr::Literal(lit, _) = e {
        assert!(matches!(lit, crate::ast::Literal::Hex(_)));
    }
}

#[test]
fn test_expr_literal_bits() {
    let e = first_expr("def f() { 0b1010; }");
    if let crate::ast::Expr::Literal(lit, _) = e {
        assert!(matches!(lit, crate::ast::Literal::Bits(_)));
    }
}

#[test]
fn test_expr_literal_true() {
    let e = first_expr("def f() { true; }");
    if let crate::ast::Expr::Literal(lit, _) = e {
        assert!(matches!(lit, crate::ast::Literal::Bool(true)));
    }
}

#[test]
fn test_expr_literal_false() {
    let e = first_expr("def f() { false; }");
    if let crate::ast::Expr::Literal(lit, _) = e {
        assert!(matches!(lit, crate::ast::Literal::Bool(false)));
    }
}

#[test]
fn test_expr_literal_string() {
    let e = first_expr(r#"def f() { "hello"; }"#);
    if let crate::ast::Expr::Literal(lit, _) = e {
        assert!(matches!(lit, crate::ast::Literal::Str(_)));
    }
}

#[test]
fn test_expr_literal_char() {
    let e = first_expr("def f() { 'x'; }");
    if let crate::ast::Expr::Literal(lit, _) = e {
        assert!(matches!(lit, crate::ast::Literal::Char('x')));
    }
}

#[test]
fn test_expr_identifier() {
    let e = first_expr("def f() { my_var; }");
    if let crate::ast::Expr::Identifier(id) = e {
        assert_eq!(id.name, "my_var");
    }
}

fn slice_from_assignment(source: &str) -> crate::ast::SliceExpr {
    let e = first_expr(source);
    if let crate::ast::Expr::Binary(b) = e {
        if let crate::ast::Expr::Slice(s) = *b.right {
            return s;
        }
    }
    panic!("expected assignment with slice on right side");
}

#[test]
fn test_slice_constant_range() {
    let s = slice_from_assignment("def f() { x = arr[0..5]; }");
    assert_eq!(s.ranges.len(), 1);
    assert!(s.ranges[0].end.is_some());
}

#[test]
fn test_slice_variable_range() {
    let s = slice_from_assignment("def f() { x = arr[i..j]; }");
    assert_eq!(s.ranges.len(), 1);
    assert!(s.ranges[0].end.is_some());
}

#[test]
fn test_slice_from_index_to_end() {
    let s = slice_from_assignment("def f() { x = arr[i..]; }");
    assert_eq!(s.ranges.len(), 1);
    assert!(s.ranges[0].end.is_none());
}

#[test]
fn test_slice_from_start_to_index() {
    let s = slice_from_assignment("def f() { x = arr[..j]; }");
    assert_eq!(s.ranges.len(), 1);
    assert!(s.ranges[0].end.is_some());
}

#[test]
fn test_func_literal() {
    let e = first_expr("def f() { g = def my_func(a of int) of int { return a+1; }; }");
    if let crate::ast::Expr::Binary(b) = e {
        assert!(matches!(b.operator, crate::ast::BinaryOp::Assign));
        assert!(matches!(*b.right, crate::ast::Expr::FuncLiteral(_)));
    } else {
        panic!("expected assignment");
    }
}

#[test]
fn test_func_type_var() {
    let program = parse("def f() { g of def(int) of int; }");
    if let crate::ast::SourceItem::FuncDefinition(func) = &program.items[0] {
        assert!(matches!(func.body[0], crate::ast::Statement::VarDecl(_)));
        if let crate::ast::Statement::VarDecl(vd) = &func.body[0] {
            assert!(matches!(vd.ty, crate::ast::TypeRef::Function { .. }));
        }
    }
}
