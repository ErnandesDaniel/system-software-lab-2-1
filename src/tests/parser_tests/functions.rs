use crate::tests::parse;

fn as_func<'a>(program: &'a crate::ast::Program, idx: usize) -> &'a crate::ast::FuncDefinition {
    if let crate::ast::SourceItem::FuncDefinition(f) = &program.items[idx] {
        f
    } else {
        panic!("item {idx} is not FuncDefinition");
    }
}

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
fn test_function_call_with_args() {
    let program = parse("import puts def main() { puts(\"hello\"); }");
    assert_eq!(program.items.len(), 2);
    if let crate::ast::SourceItem::FuncDefinition(f) = &program.items[1] {
        assert_eq!(f.signature.name.name, "main");
    }
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
    let program = parse("def a() { } def b() { } def c() { } def d() { } def e() { }");
    assert_eq!(program.items.len(), 5);
}

#[test]
fn test_decl_import_and_def() {
    let program = parse("import def puts(s of string); def main() { puts(\"hi\"); }");
    assert_eq!(program.items.len(), 2);
}

#[test]
fn test_decl_func_param_custom_type() {
    let program = parse("def foo(p of Point) { }");
    let f = as_func(&program, 0);
    if let Some(params) = &f.signature.parameters {
        assert!(matches!(&params[0].ty, Some(crate::ast::TypeRef::Custom(_))));
    }
}

#[test]
fn test_simple_assign() {
    let source = "def foo() { a = 5; }";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}
