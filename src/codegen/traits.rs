/// Trait for shared operand analysis logic between code generation backends
/// 
/// This trait provides common helper methods for analyzing IR operands,
/// used by both NASM (assembly) and LLVM IR code generators.
pub trait OperandLoader {
    /// Check if a variable name represents a temporary SSA value (t0, t1, t2, etc.)
    /// 
    /// Temporaries are compiler-generated values that don't need allocation in memory
    /// and can be used directly in SSA form.
    fn is_temp(name: &str) -> bool {
        name.len() >= 2 
            && name.starts_with('t')
            && name[1..].chars().all(|c| c.is_ascii_digit())
    }
}

/// Blanket implementation - makes OperandLoader available for all types
impl<T> OperandLoader for T {}
