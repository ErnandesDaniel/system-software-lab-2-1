use crate::tests::cross_target_tests::compile_and_run_jvm;

#[test]
fn test_jvm_runtime_return_42() {
    let output = compile_and_run_jvm("def main() of int { return 42; }");
    assert_eq!(output.status.code(), Some(42), "jvm runtime should return 42");
}
