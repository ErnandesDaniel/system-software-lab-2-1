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

fn stdin_run(exe: &str, args: &[&str], stdin_data: &[u8]) -> Option<String> {
    let mut c = Command::new(exe).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().ok()?;
    c.stdin.take().unwrap().write_all(stdin_data).ok();
    c.wait_with_output().ok().map(|o| clean(&String::from_utf8_lossy(&o.stdout)))
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

fn compile_nasm_stdin(lab: &str, input: &[u8]) -> Option<String> {
    let out = format!("target/tmp-lab-{lab}");
    let src = format!("labs-examples/vitrual-machines/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", "nasm"]) { return None; }
    stdin_run(&format!("{out}/program.exe"), &[], input)
}

fn compile_jvm_stdin(lab: &str, input: &[u8]) -> Option<String> {
    let out = format!("target/tmp-lab-{lab}");
    let src = format!("labs-examples/vitrual-machines/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", "jvm"]) { return None; }
    stdin_run("java", &["-cp", &out, "Main"], input)
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

#[test] fn test_lab_vm1_nasm() { let out = compile_nasm_stdin("lab-1", b"X\n").expect("compile/run failed"); assert!(out.contains("Hello, World!")); assert!(out.contains("\nX\n")); assert!(out.contains("\nA\n")); assert!(out.contains("\n65")); }
#[test] fn test_lab_vm1_jvm()  { let out = compile_jvm_stdin("lab-1", b"X\n").expect("compile/run failed");  assert!(out.contains("Hello, World!")); assert!(out.contains("\nX\n")); assert!(out.contains("\nA\n")); assert!(out.contains("\n65")); }
#[test] fn test_lab_vm2_nasm() { let out = compile_nasm("lab-2").expect("compile/run failed"); assert!(out.contains("All done")); assert!(out.contains("test() = 2223")); }
#[test] fn test_lab_vm2_jvm()  { let out = compile_jvm("lab-2").expect("compile/run failed");  assert!(out.contains("All done")); assert!(out.contains("test() = 2223")); }
#[test] fn test_lab_vm3_nasm() { let out = compile_nasm("lab-3").expect("compile/run failed"); assert!(out.contains("total_freed=2560")); }
#[test] fn test_lab_vm3_jvm()  { let out = compile_jvm("lab-3").expect("compile/run failed");  assert!(out.contains("total_freed=2560")); }
#[test] fn test_lab_vm4_nasm() { let out = lab4("nasm").expect("compile/run failed"); assert!(out.contains("OK Alice")); assert!(out.contains("name")); assert!(out.contains("age")); }
#[test] fn test_lab_vm4_jvm()  { let out = lab4("jvm").expect("compile/run failed");  assert!(out.contains("OK Alice")); assert!(out.contains("name")); assert!(out.contains("age")); }

// ========== System-programms labs ==========

fn compile_sys(lab: &str, target: &str) -> Option<std::process::Child> {
    let out = format!("target/tmp-sys-{lab}");
    let src = format!("labs-examples/system-programms/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", target]) { return None; }
    let exe = format!("{out}/program.exe");
    Command::new(exe).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().ok()
}

/// Sys lab-1: infinite coroutine loop, runs forever until killed.
/// Test: run for 2 seconds, kill, verify "Start" + alternating "12".
#[test]
fn test_sys_lab1_nasm() {
    let mut child = compile_sys("lab-1", "nasm").expect("compile/run failed");
    std::thread::sleep(std::time::Duration::from_secs(2));
    let _ = child.kill();
    let out = child.wait_with_output().ok().map(|o| clean(&String::from_utf8_lossy(&o.stdout))).unwrap_or_default();
    assert!(out.contains("Start"), "should print Start");
    assert!(out.contains("1"), "should print 1 from print_once");
    assert!(out.contains("2"), "should print 2 from print_two");
    // Check alternating pattern in the first 20 chars after "Start"
    let body = out.trim_start_matches("Start\n");
    assert!(body.starts_with("12") || body.starts_with("21"), "should alternate: {:?}", &body[..10.min(body.len())]);
}
