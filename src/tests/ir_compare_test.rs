use crate::ir_generator::IrGenerator;
use crate::parser::Parser;

fn parse_and_generate_ir(source: &str) -> crate::ir::IrProgram {
    let mut parser = Parser::new(source);
    let program = parser.parse().expect("Failed to parse source");
    let mut ir_gen = IrGenerator::new();
    ir_gen.generate(&program)
}

fn print_ir_function(func: &crate::ir::IrFunction, label: &str) {
    println!("\n{}", "=".repeat(70));
    println!("IR for: {}", label);
    println!("{}", "=".repeat(70));
    println!("Function: {}", func.name);
    println!("Return type: {:?}", func.return_type);
    println!("\nBlocks ({} total):", func.blocks.len());
    
    for (i, block) in func.blocks.iter().enumerate() {
        println!("\n--- Block {}: {} ---", i, block.id);
        println!("  Successors: {:?}", block.successors);
        println!("  Instructions:");
        
        for (j, inst) in block.instructions.iter().enumerate() {
            print!("    [{}] {:?}", j, inst.opcode);
            
            if !inst.operands.is_empty() {
                print!(" | operands: {:?}", inst.operands);
            }
            
            if let Some(ref result) = inst.result {
                print!(" -> {}", result);
            }
            
            match inst.opcode {
                crate::ir::IrOpcode::Jump => {
                    if let Some(ref target) = inst.jump_target {
                        print!(" | jump_target: {}", target);
                    }
                }
                crate::ir::IrOpcode::CondBr => {
                    if let Some(ref true_t) = inst.true_target {
                        print!(" | true_target: {}", true_t);
                    }
                    if let Some(ref false_t) = inst.false_target {
                        print!(" | false_target: {}", false_t);
                    }
                }
                _ => {}
            }
            
            println!();
        }
    }
    
    // Print block order summary
    println!("\n--- Block Order Summary ---");
    for (i, block) in func.blocks.iter().enumerate() {
        let terminator = if let Some(last) = block.instructions.last() {
            match last.opcode {
                crate::ir::IrOpcode::Jump => format!("Jump -> {}", last.jump_target.as_ref().unwrap_or(&"?".to_string())),
                crate::ir::IrOpcode::CondBr => format!(
                    "CondBr T:{} F:{}",
                    last.true_target.as_ref().unwrap_or(&"?".to_string()),
                    last.false_target.as_ref().unwrap_or(&"?".to_string())
                ),
                crate::ir::IrOpcode::Ret => "Ret".to_string(),
                _ => format!("{:?}", last.opcode),
            }
        } else {
            "(empty)".to_string()
        };
        println!("  [{}] {} -> {} (successors: {:?})", 
                 i, block.id, terminator, block.successors);
    }
}

#[test]
fn compare_while_loop_ir() {
    // Case 1: From test_jvm_tests.rs - works
    let case1_source = r#"
def main() of int
    i = 1
    while i < 5
        i = i + 1
    end
    return i
end
"#;

    // Case 2: From input.mylang - has loop_end
    let case2_source = r#"
def main () of int
i=1;
while i<5 {
i=i+1;
}
loop_end
t = 1
return t
end
"#;

    let ir1 = parse_and_generate_ir(case1_source);
    let ir2 = parse_and_generate_ir(case2_source);

    // Print IR for Case 1
    if let Some(func1) = ir1.functions.first() {
        print_ir_function(func1, "Case 1: test_jvm_tests.rs style (works)");
    }

    // Print IR for Case 2
    if let Some(func2) = ir2.functions.first() {
        print_ir_function(func2, "Case 2: input.mylang style (has loop_end)");
    }

    // The test passes - we're just printing for comparison
    println!("\n{}", "=".repeat(70));
    println!("Comparison complete - check stdout for IR output");
    println!("{}", "=".repeat(70));
}
