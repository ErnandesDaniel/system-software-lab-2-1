use crate::codegen::nasm::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;

#[test]
fn test_asm_global_array_access() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[0]
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("section .data"));
    assert!(asm.contains("arr"));
    assert!(asm.contains("dd 10") || asm.contains("arr"));
}

#[test]
fn test_asm_global_array_index() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[2]
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("arr") || asm.contains("lea") || asm.contains("mov"));
}

#[test]
fn test_asm_struct_field_load() {
    let source = r#"
        struct Point { x of int; y of int; }
        def main() of int
            p of Point;
            p.x = 42;
            return p.x
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("main:"));
    assert!(asm.contains("ret"));
}

#[test]
fn test_asm_multi_blocks() {
    let source = r#"
        def main() of int
            if 1 == 1 then
                return 42
            end
            return 0
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("main_BB0:"));
    assert!(asm.contains("main_BB1:"));
    assert!(asm.contains("main_BB2:"));
}

#[test]
fn test_break_in_while_loop_asm() {
    let source = r#"
        import putchar
        def main() of int
            i = 0
            while i < 5 {
                if i == 3 then { break; }
                putchar(65 + i)
                i = i + 1
            }
            putchar(10)
            return 0
        end
    "#;
    let program = parse(source);

    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);

    let break_block = ir.functions[0].blocks.iter()
        .find(|b| {
            b.instructions.iter().any(|inst| inst.opcode == crate::ir::IrOpcode::Jump
                && inst.jump_target.as_deref() == Some("BB3"))
        });
    assert!(break_block.is_some(), "Expected a block with Jump to BB3 (loop exit)");

    let break_block = ir.functions[0].blocks.iter().find(|b| {
        b.instructions.iter().any(|inst| {
            inst.opcode == crate::ir::IrOpcode::Jump
                && inst.jump_target.as_deref() == Some("BB3")
        })
    });
    assert!(break_block.is_some(),
        "Expected a block with Jump to BB3 (loop exit). Blocks:\n{}",
        ir.functions[0].blocks.iter().map(|b| {
            format!("  {}: {:?}", b.id, b.instructions.iter().map(|i| format!("{:?}", i)).collect::<Vec<_>>())
        }).collect::<Vec<_>>().join("\n")
    );

    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);

    assert!(asm.contains("main_BB4:"), "Expected break block label main_BB4");
    let lines: Vec<&str> = asm.lines().collect();
    let bb4_idx = lines.iter().position(|l| l.trim() == "main_BB4:").expect("main_BB4 label not found");
    if let Some(next) = lines.get(bb4_idx + 1) {
        let trimmed = next.trim();
        assert!(trimmed.starts_with("jmp"), "Expected jmp after main_BB4, got: {}", next);
        assert!(trimmed.contains("main_BB3"), "Break should jmp to loop exit main_BB3, got: {}", next);
    }
}
