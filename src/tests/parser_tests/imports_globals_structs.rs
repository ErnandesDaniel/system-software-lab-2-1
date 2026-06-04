use crate::tests::parse;

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
