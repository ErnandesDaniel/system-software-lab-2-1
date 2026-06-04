/// Integration tests for VM labs.
/// Runs the exact commands from each lab's README.md,
/// then verifies expected output.
///
/// Note: these tests call `cargo run` which holds a lock on the build dir.
/// Run with: `cargo test test_lab_vm -- --test-threads=1`

use std::io::Write;
use std::process::{Command, Stdio};

fn cargo(args: &[&str]) -> bool {
    Command::new("cargo").args(["run", "--quiet", "--"]).args(args).status().map(|s| s.success()).unwrap_or(false)
}

fn clean(s: &str) -> String {
    s.replace("\r\n", "\n").chars().filter(|&c| c != '\0').collect()
}

fn compile_nasm(lab: &str) -> Option<String> {
    let out = format!("target/tmp-lab-{lab}");
    let src = format!("labs-examples/vitrual-machines/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", "nasm"]) { return None; }
    let o = Command::new(format!("{out}/program.exe")).output().ok()?;
    Some(clean(&String::from_utf8_lossy(&o.stdout)))
}

fn compile_jvm(lab: &str) -> Option<String> {
    let out = format!("target/tmp-lab-{lab}");
    let src = format!("labs-examples/vitrual-machines/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", "jvm"]) { return None; }
    let o = Command::new("java").args(["-cp", &out, "Main"]).output().ok()?;
    Some(clean(&String::from_utf8_lossy(&o.stdout)))
}

fn lab4(target: &str) -> Option<String> {
    let input = b"create name Alice\ncreate age 30\nget name\nlist\nexit\n";
    let out = "target/tmp-lab4";
    let src = "labs-examples/vitrual-machines/lab-4/input.mylang";
    if !cargo(&[src, "-o", out, "-t", target]) { return None; }
    let mut c = if target == "nasm" {
        Command::new(format!("{out}/program.exe")).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().ok()?
    } else {
        Command::new("java").args(["-cp", out, "RuntimeStub"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().ok()?
    };
    c.stdin.take().unwrap().write_all(input).ok();
    let o = c.wait_with_output().ok()?;
    Some(clean(&String::from_utf8_lossy(&o.stdout)))
}

#[test] fn test_lab_vm1_nasm() { let out = compile_nasm("lab-1").expect("compile/run failed"); assert!(out.contains("Hello, World!")); }
#[test] fn test_lab_vm1_jvm()  { let out = compile_jvm("lab-1").expect("compile/run failed");  assert!(out.contains("Hello, World!")); }
#[test] fn test_lab_vm2_nasm() { let out = compile_nasm("lab-2").expect("compile/run failed"); assert_eq!(out.contains("All done"), true); }
#[test] fn test_lab_vm2_jvm()  { let out = compile_jvm("lab-2").expect("compile/run failed");  assert_eq!(out.contains("All done"), true); }
#[test] fn test_lab_vm3_nasm() { let out = compile_nasm("lab-3").expect("compile/run failed"); assert!(out.contains("total_freed=2560")); }
#[test] fn test_lab_vm3_jvm()  { let out = compile_jvm("lab-3").expect("compile/run failed");  assert!(out.contains("total_freed=2560")); }
#[test] fn test_lab_vm4_nasm() { let out = lab4("nasm").expect("compile/run failed"); assert!(out.contains("OK Alice")); }
#[test] fn test_lab_vm4_jvm()  { let out = lab4("jvm").expect("compile/run failed");  assert!(out.contains("OK Alice")); }
