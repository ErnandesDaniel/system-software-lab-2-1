#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    fn parse(source: &str) -> crate::ast::Program {
        let mut parser = Parser::new(source);
        parser.parse().unwrap()
    }

    #[test]
    fn test_function_with_params() {
        let source = "def add(a of int, b of int) return a + b; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_function_with_return_type() {
        let source = "def foo() of int return 1; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_if_statement() {
        let source = "def foo() if x then return 1; end end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_array_type() {
        let source = "def foo(arr int array[10]) end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_while_loop() {
        let source = "def foo() while x < 10 x = x + 1; end end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_extern_declaration() {
        let source = "extern def print(msg of string) end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_binary_expressions() {
        let source = "def foo() x = 1 + 2 * 3; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_unary_expressions() {
        let source = "def foo() x = -5; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_function_call() {
        let source = "def foo() x = bar(1, 2); end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_break_statement() {
        let source = "def foo() break; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_repeat_loop() {
        let source = "def foo() do x = x + 1; while x < 10; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_block_statement() {
        let source = "def foo() begin x = 1; y = 2; end end";
        let result = {
            let mut parser = crate::parser::Parser::new(source);
            parser.parse()
        };
        eprintln!("Result: {:?}", result);
        let program = result.unwrap();
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_multiple_functions() {
        let source = "def foo() x = 1; end def bar() y = 2; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 2);
    }

    #[test]
    fn test_comparison_operators() {
        let source = "def foo() x = 1 < 2; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_logical_operators() {
        let source = "def foo() x = true && false; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_string_literal() {
        let source = "def foo() x = \"hello\"; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_char_literal() {
        let source = "def foo() x = 'a'; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_bool_literal() {
        let source = "def foo() x = true; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_hex_literal() {
        let source = "def foo() x = 0xFF; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_bits_literal() {
        let source = "def foo() x = 0b1010; end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }

    #[test]
    fn test_parenthesized_expr() {
        let source = "def foo() x = (1 + 2); end";
        let program = parse(source);
        assert_eq!(program.items.len(), 1);
    }
}
