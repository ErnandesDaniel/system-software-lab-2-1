/// Integration tests for VM labs.
/// Runs the exact commands from each lab's README.md,
/// then verifies expected output.
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;

static CARGO_LOCK: Mutex<()> = Mutex::new(());

fn cargo(args: &[&str]) -> bool {
    let _lock = CARGO_LOCK.lock().unwrap();
    Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn clean(s: &str) -> String {
    s.replace("\r\n", "\n").chars().filter(|&c| c != '\0').collect()
}

fn stdin_run(exe: &str, args: &[&str], stdin_data: &[u8]) -> Option<String> {
    let mut c = Command::new(exe)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .ok()?;
    c.stdin.take().unwrap().write_all(stdin_data).ok();
    c.wait_with_output()
        .ok()
        .map(|o| clean(&String::from_utf8_lossy(&o.stdout)))
}

fn compile_nasm(lab: &str) -> Option<String> {
    let out = format!("target/tmp-{lab}");
    let src = format!("labs-examples/vitrual-machines/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", "nasm"]) {
        return None;
    }
    let o = Command::new(format!("{out}/program.exe")).output().ok()?;
    Some(clean(&String::from_utf8_lossy(&o.stdout)))
}

fn compile_jvm(lab: &str) -> Option<String> {
    let out = format!("target/tmp-{lab}");
    let src = format!("labs-examples/vitrual-machines/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", "jvm"]) {
        return None;
    }
    let o = Command::new("java").args(["-cp", &out, "Main"]).output().ok()?;
    Some(clean(&String::from_utf8_lossy(&o.stdout)))
}

fn compile_nasm_stdin(lab: &str, input: &[u8]) -> Option<String> {
    let out = format!("target/tmp-{lab}");
    let src = format!("labs-examples/vitrual-machines/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", "nasm"]) {
        return None;
    }
    stdin_run(&format!("{out}/program.exe"), &[], input)
}

fn compile_jvm_stdin(lab: &str, input: &[u8]) -> Option<String> {
    let out = format!("target/tmp-{lab}");
    let src = format!("labs-examples/vitrual-machines/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", "jvm"]) {
        return None;
    }
    stdin_run("java", &["-cp", &out, "Main"], input)
}

fn lab4(target: &str) -> Option<String> {
    let input = b"create name Alice\ncreate age 30\nget name\nlist\nexit\n";
    let out = format!("target/tmp-lab4-{target}");
    let src = "labs-examples/vitrual-machines/lab-4/input.mylang";
    if !cargo(&[src, "-o", &out, "-t", target]) {
        return None;
    }
    let mut c = if target == "nasm" {
        Command::new(format!("{out}/program.exe"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .ok()?
    } else {
        Command::new("java")
            .args(["-cp", &out, "RuntimeStub"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .ok()?
    };
    c.stdin.take().unwrap().write_all(input).ok();
    let o = c.wait_with_output().ok()?;
    Some(clean(&String::from_utf8_lossy(&o.stdout)))
}

#[test]
fn test_lab_vm1_nasm() {
    let out = compile_nasm_stdin("lab-1", b"X\n").expect("compile/run failed");
    assert!(out.contains("Hello, World!"));
    assert!(out.contains("\nX\n"));
    assert!(out.contains("\nA\n"));
    assert!(out.contains("\n65"));
}
#[test]
fn test_lab_vm1_jvm() {
    let out = compile_jvm_stdin("lab-1", b"X\n").expect("compile/run failed");
    assert!(out.contains("Hello, World!"));
    assert!(out.contains("\nX\n"));
    assert!(out.contains("\nA\n"));
    assert!(out.contains("\n65"));
}
#[test]
fn test_lab_vm2_nasm() {
    let out = compile_nasm("lab-2").expect("compile/run failed");
    assert!(out.contains("All done"));
    assert!(out.contains("test() = 2223"));
}
#[test]
fn test_lab_vm2_jvm() {
    let out = compile_jvm("lab-2").expect("compile/run failed");
    assert!(out.contains("All done"));
    assert!(out.contains("test() = 2223"));
}
#[test]
fn test_lab_vm3_nasm() {
    let out = compile_nasm("lab-3").expect("compile/run failed");
    assert!(out.contains("total_freed=2560"));
}
#[test]
fn test_lab_vm3_jvm() {
    let out = compile_jvm("lab-3").expect("compile/run failed");
    assert!(out.contains("total_freed=2560"));
}
#[test]
fn test_lab_vm4_nasm() {
    let out = lab4("nasm").expect("compile/run failed");
    assert!(out.contains("OK Alice"));
    assert!(out.contains("name"));
    assert!(out.contains("age"));
}
#[test]
fn test_lab_vm4_jvm() {
    let out = lab4("jvm").expect("compile/run failed");
    assert!(out.contains("OK Alice"));
    assert!(out.contains("name"));
    assert!(out.contains("age"));
}

// ========== System-programms labs ==========

fn compile_sys(lab: &str, target: &str) -> Option<std::process::Child> {
    let out = format!("target/tmp-sys-{lab}");
    let src = format!("labs-examples/system-programms/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", target]) {
        return None;
    }
    let exe = format!("{out}/program.exe");
    Command::new(exe)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()
}

/// Sys lab-1: infinite coroutine loop, runs forever until killed.
fn test_sys_lab1(target: &str) {
    let out = format!("target/tmp-sys-lab1-{target}");
    let src = "labs-examples/system-programms/lab-1/input.mylang";
    if !cargo(&[src, "-o", &out, "-t", target]) {
        panic!("compile failed");
    }
    let exe = if target == "nasm" {
        format!("{out}/program.exe")
    } else {
        "java".to_string()
    };
    let args: &[&str] = if target == "nasm" { &[] } else { &["-cp", &out, "Main"] };
    let mut child = Command::new(&exe)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn failed");
    std::thread::sleep(std::time::Duration::from_secs(2));
    let _ = child.kill();
    let out = child
        .wait_with_output()
        .ok()
        .map(|o| clean(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or_default();
    assert!(out.contains("Start"), "should print Start");
    assert!(out.contains("1"), "should print 1");
    assert!(out.contains("2"), "should print 2");
    let body = out.trim_start_matches("Start\n");
    assert!(
        body.starts_with("12") || body.starts_with("21"),
        "should alternate: {:?}",
        &body[..10.min(body.len())]
    );
}

#[test]
fn test_sys_lab1_nasm() {
    test_sys_lab1("nasm");
}
#[test]
fn test_sys_lab1_jvm() {
    test_sys_lab1("jvm");
}

/// Sys lab-1 metrics: scheduling simulator with struct globals.
fn test_sys_lab1_metrics(target: &str) {
    let out = format!("target/tmp-sys-lab1-metrics-{target}");
    let src = "labs-examples/system-programms/lab-1/metrics.mylang";
    if !cargo(&[src, "-o", &out, "-t", target]) {
        panic!("compile failed");
    }
    let exe = if target == "nasm" {
        format!("{out}/program.exe")
    } else {
        "java".to_string()
    };
    let args: &[&str] = if target == "nasm" { &[] } else { &["-cp", &out, "Main"] };
    let output = Command::new(&exe)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn failed");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    let stdout = clean(&String::from_utf8_lossy(&output.stdout));
    assert!(stdout.contains("Scheduling (Var 19) ==="));
    assert!(stdout.contains("=== Done ==="));
    assert!(stdout.contains("RR(2):"));
    assert!(stdout.contains("SRT:"));
    assert!(stdout.contains("Avg turn:"));
    assert!(stdout.contains("Avg wait:"));
}

#[test]
fn test_sys_lab1_metrics_nasm() {
    test_sys_lab1_metrics("nasm");
}
#[test]
fn test_sys_lab1_metrics_jvm() {
    test_sys_lab1_metrics("jvm");
}

// ========== Coroutine + file I/O regression tests ==========

fn compile_nasm_src(src: &str, tmp_dir: &str) -> bool {
    let out = format!("target/{tmp_dir}");
    if !cargo(&[src, "-o", &out, "-t", "nasm"]) {
        return false;
    }
    let output = Command::new(format!("{out}/program.exe"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn failed");
    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("STDOUT: [{stdout}]");
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("STDERR: [{stderr}]");
    }
    output.status.success()
}

fn write_test_src(name: &str, content: &str) -> String {
    let dir = format!("target/tmp-coro-{name}");
    let _ = fs::create_dir_all(&dir);
    let path = format!("{dir}/input.mylang");
    fs::write(&path, content).expect("write test src");
    path
}

/// Regression: regular (non-coroutine) fopen + fgetc.
#[test]
fn test_regular_fopen_fgetc() {
    let src = write_test_src("regular_fopen_fgetc", r#"import puts;
import fopen;
import fgetc;
import fclose;

def main() of int {
    gf = fopen("labs-examples/system-programms/lab-2/csv-data/people.csv", "r");
    if (gf == 0) { puts("FAIL"); return 1; }
    c = fgetc(gf);
    fclose(gf);
    puts("=== DONE ===");
    return 0;
}
"#);
    assert!(compile_nasm_src(&src, "regular-fopen-fgetc"), "regular fopen+fgetc should pass");
}

/// Regression: coroutine fopen + yield then fgetc.
#[test]
fn test_coro_fopen_yield_fgetc() {
    let src = write_test_src("fopen_yield_fgetc", r#"import puts;
import fopen;
import fgetc;
import fclose;
import resume_coroutine;
import coro_init;
import create_coroutine;

coroutine reader() of int {
    gf = fopen("labs-examples/system-programms/lab-2/csv-data/people.csv", "r");
    if (gf == 0) { return 0; }
    yield;
    c = fgetc(gf);
    fclose(gf);
    return c;
}
def main() of int {
    coro_init();
    r = resume_coroutine(0);
    r = resume_coroutine(0);
    puts("=== DONE ===");
    return 0;
}
"#);
    assert!(compile_nasm_src(&src, "coro-fopen-yield-fgetc"), "coroutine fopen yield fgetc should pass");
}

/// Regression: coroutine fopen + yield + read_line wrapper (reproduces lab 2).
#[test]
fn test_coro_fopen_yield_read_line() {
    let src = write_test_src("fopen_yield_read_line", r#"import puts;
import fopen;
import fgetc;
import fclose;
import resume_coroutine;
import coro_init;
import create_coroutine;

def read_line(f of string) of int { return fgetc(f); }

coroutine reader() of int {
    gf = fopen("labs-examples/system-programms/lab-2/csv-data/people.csv", "r");
    if (gf == 0) { return 0; }
    yield;
    c = read_line(gf);
    fclose(gf);
    return c;
}
def main() of int {
    coro_init();
    r = resume_coroutine(0);
    r = resume_coroutine(0);
    puts("=== DONE ===");
    return 0;
}
"#);
    assert!(compile_nasm_src(&src, "coro-fopen-yield-read-line"), "coroutine fopen+yield+read_line should pass");
}
