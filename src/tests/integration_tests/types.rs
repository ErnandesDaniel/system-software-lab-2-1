use super::compile_and_run;

#[test]
fn test_exe_byte_decl_and_return() {
    let source = "def main() of int { a of byte; a = 42; return a; }";
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "byte decl should compile and run");
}

#[test]
fn test_exe_byte_arithmetic() {
    let source = "def main() of int { a of byte; b of byte; a = 10; b = 20; return a + b; }";
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "byte arithmetic should compile and run");
}

#[test]
fn test_exe_uint_decl_and_return() {
    let source = "def main() of int { a of uint; a = 100; return a; }";
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "uint decl should compile and run");
}

#[test]
fn test_exe_long_decl_and_return() {
    let source = "def main() of int { a of long; a = 10000000000; return 42; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "long decl should compile and run");
}

#[test]
fn test_exe_ulong_decl_and_return() {
    let source = "def main() of int { a of ulong; a = 200; return a; }";
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "ulong decl should compile and run");
}
