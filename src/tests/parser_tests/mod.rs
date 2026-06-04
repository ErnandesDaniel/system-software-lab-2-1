use crate::tests::parse;

#[test]
fn test_function_with_params() {
    let program = parse("def add(a of int, b of int) { return a + b; }");
    assert_eq!(program.items.len(), 1);
    if let crate::ast::SourceItem::FuncDefinition(f) = &program.items[0] {
        assert_eq!(f.signature.name.name, "add");
        if let Some(params) = &f.signature.parameters {
            assert_eq!(params.len(), 2);
            assert_eq!(params[0].name.name, "a");
            assert_eq!(params[1].name.name, "b");
        } else {
            panic!("Expected parameters");
        }
    } else {
        panic!("Expected FuncDefinition");
    }
}

#[test]
fn test_function_with_return_type() {
    let program = parse("def foo() of int { return 1; }");
    assert_eq!(program.items.len(), 1);
    if let crate::ast::SourceItem::FuncDefinition(f) = &program.items[0] {
        assert_eq!(f.signature.name.name, "foo");
        assert!(f.signature.return_type.is_some(), "Expected return type");
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
fn test_import_declaration() {
    let program = parse("import def print(msg of string);");
    assert_eq!(program.items.len(), 1);
    if let crate::ast::SourceItem::FuncDeclaration(d) = &program.items[0] {
        assert_eq!(d.signature.name.name, "print");
    }
}

#[test]
fn test_import_short_form() {
    let program = parse("import puts");
    assert_eq!(program.items.len(), 1);
    if let crate::ast::SourceItem::FuncDeclaration(d) = &program.items[0] {
        assert_eq!(d.signature.name.name, "puts");
        assert!(d.signature.parameters.is_none(), "Short form has no params");
    }
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
fn test_begin_end_block() {
    let program = parse("def foo() { { x = 1; } }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_array_indexed_assignment() {
    let program = parse("def foo() { arr[0] = 1; }");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_function_call_with_args() {
    let program = parse("import puts def main() { puts(\"hello\"); }");
    assert_eq!(program.items.len(), 2);
    if let crate::ast::SourceItem::FuncDefinition(f) = &program.items[1] {
        assert_eq!(f.signature.name.name, "main");
    }
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
fn test_multiple_functions() {
    let source = r#"
        def add(a of int, b of int) of int {
            return a + b;
        }
        def main() of int {
            return add(1, 2);
        }
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 2);
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
fn test_import_with_params_and_return() {
    let source = "import def map_put_jvm(name of string, value of string) of int;";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_int() {
    let source = "global counter of int = 0;";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_string() {
    let source = "global name of string = \"hello\";";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_array_of_int() {
    let source = "global arr of int[3] = [1, 2, 3];";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_no_init() {
    let source = "global counter of int;";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_with_import_and_def() {
    let source = r#"
        global counter of int = 0;
        import puts
        def main() of int {
            return counter;
        }
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 3);
}

#[test]
fn test_struct_definition() {
    let source = r#"
        struct Point {
            x of int;
            y of int;
        }
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_struct() {
    let source = r#"
        struct Point {
            x of int;
            y of int;
        }
        global p of Point;
        def main() of int {
            return p.x;
        }
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 3);
}

#[test]
fn test_field_access() {
    let source = r#"
        def main() of int {
            return a.b;
        }
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_simple_assign() {
    let source = "def foo() { a = 5; }";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

// ──────────────────────────────────────────────
// Parser error tests
// ──────────────────────────────────────────────

fn parse_err(source: &str) -> crate::CompilerError {
    let mut parser = crate::parser::Parser::new(source);
    parser.parse().unwrap_err()
}

#[test]
fn test_parse_error_invalid_char() {
    let err = parse_err("@");
    assert!(matches!(err, crate::CompilerError::Parse(_)));
}
#[test]
fn test_parse_error_unexpected_end_of_input() {
    let err = parse_err("def foo(");
    assert!(matches!(&err, crate::CompilerError::Parse(msg) if msg == "Unexpected end of input"));
}
#[test]
fn test_parse_error_unclosed_block() {
    let err = parse_err("def foo() {");
    assert!(matches!(&err, crate::CompilerError::Parse(msg) if msg == "Unexpected end of input"));
}
#[test]
fn test_parse_error_missing_rparen() {
    let err = parse_err("def foo(x of int { }");
    assert!(matches!(err, crate::CompilerError::Parse(_)));
}
#[test]
fn test_parse_error_bare_number() {
    let program = parse("42");
    assert!(program.items.is_empty());
}
#[test]
fn test_parse_error_junk_after_def() {
    let err = parse_err("def 123() {}");
    assert!(matches!(err, crate::CompilerError::Parse(_)));
}
#[test]
fn test_parse_error_array_no_base() {
    let err = parse_err("def foo() { x of array[5]; }");
    assert!(matches!(err, crate::CompilerError::Parse(_)));
}
#[test]
fn test_parse_error_array_no_size() {
    let err = parse_err("def foo() { x of int[]; }");
    assert!(matches!(&err, crate::CompilerError::Parse(msg) if msg.contains("requires a size")));
}

// ──────────────────────────────────────────────
// Expression parsing tests
// ──────────────────────────────────────────────

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

// ──────────────────────────────────────────────
// Statement parsing tests
// ──────────────────────────────────────────────

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

// ──────────────────────────────────────────────
// Top-level declaration tests
// ──────────────────────────────────────────────

fn as_func<'a>(program: &'a crate::ast::Program, idx: usize) -> &'a crate::ast::FuncDefinition {
    if let crate::ast::SourceItem::FuncDefinition(f) = &program.items[idx] {
        f
    } else {
        panic!("item {idx} is not FuncDefinition");
    }
}

#[test]
fn test_decl_func_no_params() {
    let program = parse("def main() { }");
    let f = as_func(&program, 0);
    assert_eq!(f.signature.name.name, "main");
    assert!(f.signature.parameters.is_none() || f.signature.parameters.as_ref().unwrap().is_empty());
}
#[test]
fn test_decl_func_one_param() {
    let program = parse("def foo(x of int) { }");
    let f = as_func(&program, 0);
    if let Some(params) = &f.signature.parameters {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name.name, "x");
    }
}
#[test]
fn test_decl_func_three_params() {
    let program = parse("def foo(a of int, b of int, c of int) { }");
    let f = as_func(&program, 0);
    if let Some(params) = &f.signature.parameters {
        assert_eq!(params.len(), 3);
    }
}
#[test]
fn test_decl_func_with_return_type() {
    let program = parse("def foo() of int { return 0; }");
    let f = as_func(&program, 0);
    assert!(f.signature.return_type.is_some());
}
#[test]
fn test_decl_func_no_return_type() {
    let program = parse("def foo() { }");
    let f = as_func(&program, 0);
    assert!(f.signature.return_type.is_none());
}
#[test]
fn test_decl_func_with_bool_return() {
    let program = parse("def foo() of bool { return true; }");
    assert_eq!(program.items.len(), 1);
}
#[test]
fn test_decl_func_with_string_return() {
    let program = parse("def foo() of string { return \"hello\"; }");
    assert_eq!(program.items.len(), 1);
}
#[test]
fn test_decl_import_short() {
    let program = parse("import puts");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::FuncDeclaration(_)));
}
#[test]
fn test_decl_import_full() {
    let program = parse("import def foo(x of int) of int;");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::FuncDeclaration(_)));
}
#[test]
fn test_decl_import_no_params() {
    let program = parse("import def foo() of int;");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::FuncDeclaration(_)));
}
#[test]
fn test_decl_import_void_return() {
    let program = parse("import def foo(x of int);");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::FuncDeclaration(_)));
}
#[test]
fn test_decl_global_int() {
    let program = parse("global x of int = 5;");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::GlobalDecl(_)));
}
#[test]
fn test_decl_global_no_init() {
    let program = parse("global x of int;");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::GlobalDecl(_)));
}
#[test]
fn test_decl_global_string() {
    let program = parse(r#"global name of string = "world";"#);
    assert!(matches!(&program.items[0], crate::ast::SourceItem::GlobalDecl(_)));
}
#[test]
fn test_decl_global_bool() {
    let program = parse("global flag of bool = true;");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::GlobalDecl(_)));
}
#[test]
fn test_decl_global_array() {
    let program = parse("global arr of int[3] = [1, 2, 3];");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::GlobalDecl(_)));
}
#[test]
fn test_decl_struct_empty() {
    let program = parse("struct Empty { }");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::StructDef(_)));
}
#[test]
fn test_decl_struct_one_field() {
    let program = parse("struct Point { x of int; }");
    if let crate::ast::SourceItem::StructDef(s) = &program.items[0] {
        assert_eq!(s.fields.len(), 1);
        assert_eq!(s.fields[0].name.name, "x");
    }
}
#[test]
fn test_decl_struct_multi_field() {
    let program = parse("struct Point { x of int; y of int; z of int; }");
    if let crate::ast::SourceItem::StructDef(s) = &program.items[0] {
        assert_eq!(s.fields.len(), 3);
    }
}
#[test]
fn test_decl_struct_mixed_types() {
    let program = parse("struct Person { name of string; age of int; active of bool; }");
    assert_eq!(program.items.len(), 1);
}
#[test]
fn test_decl_coroutine() {
    let program = parse("coroutine worker() of int { yield; return 0; }");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::CoroutineDef(_)));
}
#[test]
fn test_decl_coroutine_void() {
    let program = parse("coroutine worker() { yield; }");
    assert!(matches!(&program.items[0], crate::ast::SourceItem::CoroutineDef(_)));
}
#[test]
fn test_decl_mixed_order() {
    let program = parse("import puts\nstruct P { x of int; }\nglobal g of int = 0;\ndef main() { }");
    assert_eq!(program.items.len(), 4);
}
#[test]
fn test_decl_many_functions() {
    let program = parse(
        "def a() { } def b() { } def c() { } def d() { } def e() { }",
    );
    assert_eq!(program.items.len(), 5);
}
#[test]
fn test_decl_import_and_def() {
    let program = parse("import def puts(s of string); def main() { puts(\"hi\"); }");
    assert_eq!(program.items.len(), 2);
}
#[test]
fn test_decl_many_globals() {
    let src = (0..10)
        .map(|i| format!("global g{i} of int = {i};"))
        .collect::<Vec<_>>()
        .join("\n");
    let program = parse(&src);
    assert_eq!(program.items.len(), 10);
}
#[test]
fn test_decl_struct_field_custom_type() {
    let program = parse("struct A { b of B; }");
    if let crate::ast::SourceItem::StructDef(s) = &program.items[0] {
        assert!(matches!(&s.fields[0].ty, crate::ast::TypeRef::Custom(_)));
    }
}
#[test]
fn test_decl_func_param_custom_type() {
    let program = parse("def foo(p of Point) { }");
    let f = as_func(&program, 0);
    if let Some(params) = &f.signature.parameters {
        assert!(matches!(&params[0].ty, Some(crate::ast::TypeRef::Custom(_))));
    }
}



// ──────────────────────────────────────────────
// Chained / complex expression tests
// ──────────────────────────────────────────────

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

