use crate::tests::parse;

#[test]
fn test_function_with_params() {
    let program = parse("def add(a of int, b of int) return a + b; end");
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
    let program = parse("def foo() of int return 1; end");
    assert_eq!(program.items.len(), 1);
    if let crate::ast::SourceItem::FuncDefinition(f) = &program.items[0] {
        assert_eq!(f.signature.name.name, "foo");
        assert!(f.signature.return_type.is_some(), "Expected return type");
    }
}

#[test]
fn test_if_statement() {
    let program = parse("def foo() if x then return 1; end end");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_while_loop() {
    let program = parse("def foo() x = 1; while x < 10 { x = x + 1; } loop_end end");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_import_declaration() {
    let program = parse("import def print(msg of string) end");
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
    let program = parse("def foo() x = 1 + 2 * 3; end");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_string_literal_assignment() {
    let program = parse("def foo() s = \"hello\"; end");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_begin_end_block() {
    let program = parse("def foo() begin x = 1; end end");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_array_indexed_assignment() {
    let program = parse("def foo() arr[0] = 1; end");
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_function_call_with_args() {
    let program = parse("import puts def main() puts(\"hello\"); end");
    assert_eq!(program.items.len(), 2);
    if let crate::ast::SourceItem::FuncDefinition(f) = &program.items[1] {
        assert_eq!(f.signature.name.name, "main");
    }
}

#[test]
fn test_nested_while_loops() {
    let source = r#"
        def foo()
        i = 0;
        while i < 3 {
            j = 0;
            while j < 2 {
                j = j + 1;
            }
            loop_end
            i = i + 1;
        }
        loop_end
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_if_with_else() {
    let source = r#"
        def foo(x of int) of int
        if x > 0 then
            return 1
        else
            return 0
        end
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        def add(a of int, b of int) of int
            return a + b
        end
        def main() of int
            return add(1, 2)
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 2);
}

#[test]
fn test_multiple_statements_in_sequence() {
    let source = r#"
        def main() of int
            a = 1;
            b = 2;
            c = a + b;
            return c
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_import_with_params_and_return() {
    let source = "import def map_put_jvm(name of string, value of string) of int end";
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
        def main() of int
            return counter
        end
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
        def main() of int
            return p.x
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 3);
}

#[test]
fn test_field_access() {
    let source = r#"
        def main() of int
            return a.b
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_simple_assign() {
    let source = "def foo() a = 5; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}
