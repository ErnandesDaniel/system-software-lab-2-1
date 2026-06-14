/// Integration tests for VM labs.
/// Runs the exact commands from each lab's README.md,
/// then verifies expected output.
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

static CARGO_LOCK: Mutex<()> = Mutex::new(());

fn exe_name() -> &'static str {
    if cfg!(target_os = "linux") {
        "program"
    } else {
        "program.exe"
    }
}

fn cargo(args: &[&str]) -> bool {
    let _lock = CARGO_LOCK.lock().unwrap();
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "--quiet", "--"]);
    cmd.args(args);
    if cfg!(target_os = "linux") {
        cmd.args(["--os", "linux"]);
    }
    cmd.status().map(|s| s.success()).unwrap_or(false)
}

fn has_java() -> bool {
    Command::new("java").arg("-version").output().is_ok()
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
    let o = Command::new(format!("{out}/{}", exe_name())).output().ok()?;
    Some(clean(&String::from_utf8_lossy(&o.stdout)))
}

fn compile_jvm(lab: &str) -> Option<String> {
    if !has_java() {
        return None;
    }
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
    stdin_run(&format!("{out}/{}", exe_name()), &[], input)
}

fn compile_jvm_stdin(lab: &str, input: &[u8]) -> Option<String> {
    if !has_java() {
        return None;
    }
    let out = format!("target/tmp-{lab}");
    let src = format!("labs-examples/vitrual-machines/{lab}/input.mylang");
    if !cargo(&[&src, "-o", &out, "-t", "jvm"]) {
        return None;
    }
    stdin_run("java", &["-cp", &out, "Main"], input)
}

fn lab4(target: &str) -> Option<String> {
    if target == "jvm" && !has_java() {
        return None;
    }
    let input = b"create name Alice\ncreate age 30\nget name\nlist\nexit\n";
    let out = format!("target/tmp-lab4-{target}");
    let src = "labs-examples/vitrual-machines/lab-4/input.mylang";
    if !cargo(&[src, "-o", &out, "-t", target]) {
        return None;
    }
    let mut c = if target == "nasm" {
        Command::new(format!("{out}/{}", exe_name()))
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
#[ignore = "NASM codegen: arrays of closures with captures crash (access violation)"]
fn test_lab_vm2_nasm() {
    let out = compile_nasm("lab-2").expect("compile/run failed");
    assert!(out.contains("All done"));
    assert!(out.contains("test() = 3433"));
}
#[test]
fn test_lab_vm2_jvm() {
    let out = compile_jvm("lab-2").expect("compile/run failed");
    assert!(out.contains("All done"));
    assert!(out.contains("test() = 3433"));
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

fn compile_sys_file_timeout(file: &str, timeout_secs: u64) -> Option<String> {
    let out = "target/tmp-sys";
    let src = format!("labs-examples/system-programms/{file}");
    if !cargo(&[&src, "-o", out, "-t", "nasm"]) {
        return None;
    }
    let mut child = Command::new(format!("{out}/{}", exe_name()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    let mut stdout = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        use std::io::Read;
        let mut buf = Vec::new();
        let _ = stdout.read_to_end(&mut buf);
        buf
    });

    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    break;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => return None,
        }
    }
    let _ = child.wait();
    let out_bytes = reader.join().ok()?;
    Some(clean(&String::from_utf8_lossy(&out_bytes)))
}

// ========== System-programms labs ==========

#[cfg(target_os = "linux")]
fn compile_sys_file(file: &str) -> Option<String> {
    let out = "target/tmp-sys";
    let src = format!("labs-examples/system-programms/{file}");
    if !cargo(&[&src, "-o", out, "-t", "nasm"]) {
        return None;
    }
    let o = Command::new(format!("{out}/{}", exe_name())).output().ok()?;
    Some(clean(&String::from_utf8_lossy(&o.stdout)))
}

// --- sys-lab-1: input.mylang (coroutine demo, RR, 2 workers) ---

#[test]
#[cfg(target_os = "linux")]
fn test_sys_lab1_input_nasm() {
    let out = compile_sys_file_timeout("lab-1/input.mylang", 3).expect("compile/run failed");
    assert!(out.contains("Start"), "should print Start");
    assert!(out.len() > 100, "should produce output (got {} bytes)", out.len());
    assert!(out.contains('1'), "worker1 should print 1");
    assert!(out.contains('2'), "worker2 should print 2");
}

// --- sys-lab-1: metrics-rr.mylang (RR scheduler demo, 3 workers) ---

#[test]
#[cfg(target_os = "linux")]
fn test_sys_lab1_metrics_rr_nasm() {
    let out = compile_sys_file_timeout("lab-1/metrics-rr.mylang", 3).expect("compile/run failed");
    assert!(out.contains("RR"), "should print RR title");
    assert!(out.len() > 100, "should produce output (got {} bytes)", out.len());
    assert!(out.contains('A'), "worker A should print");
    assert!(out.contains('B'), "worker B should print");
    assert!(out.contains('C'), "worker C should print");
}

// --- sys-lab-1: metrics-srt.mylang (SRT scheduler demo, 3 workers) ---

#[test]
#[cfg(target_os = "linux")]
fn test_sys_lab1_metrics_srt_nasm() {
    let out = compile_sys_file_timeout("lab-1/metrics-srt.mylang", 3).expect("compile/run failed");
    assert!(out.contains("SRT"), "should print SRT title");
    assert!(out.len() > 100, "should produce output (got {} bytes)", out.len());
    assert!(out.contains('A'), "worker A should print");
    assert!(out.contains('B'), "worker B should print");
    assert!(out.contains('C'), "worker C should print");
}

// ========== sys-lab-3: Ext3 FTP Server ==========

/// Build a minimal ext3 filesystem image for testing.
fn create_test_ext3(path: &str) -> std::io::Result<()> {
    let block_size: u32 = 1024;
    let blocks_total: u32 = 256;
    let inodes_total: u32 = 32;
    let inode_size: u32 = 128;

    let bgdt_block_bitmap = 3u32;
    let bgdt_inode_bitmap = 4u32;
    let bgdt_inode_table = 5u32;
    let root_data_block = 9u32;
    let subdir_data_block = 10u32;
    let file_data_block = 11u32;
    let subdir_file_block = 12u32;
    let last_used_block = 12u32;

    let mut img = vec![0u8; (blocks_total * block_size) as usize];

    let sb = &mut img[1024..2048];
    sb[0..4].copy_from_slice(&inodes_total.to_le_bytes());
    sb[4..8].copy_from_slice(&blocks_total.to_le_bytes());
    sb[20..24].copy_from_slice(&1u32.to_le_bytes());
    sb[24..28].copy_from_slice(&0u32.to_le_bytes());
    sb[32..36].copy_from_slice(&blocks_total.to_le_bytes());
    sb[40..44].copy_from_slice(&inodes_total.to_le_bytes());
    sb[56..58].copy_from_slice(&0xEF53u16.to_le_bytes());
    sb[88..90].copy_from_slice(&(inode_size as u16).to_le_bytes());

    let bgdt = &mut img[2048..2080];
    bgdt[0..4].copy_from_slice(&bgdt_block_bitmap.to_le_bytes());
    bgdt[4..8].copy_from_slice(&bgdt_inode_bitmap.to_le_bytes());
    bgdt[8..12].copy_from_slice(&bgdt_inode_table.to_le_bytes());
    bgdt[12..14].copy_from_slice(&((blocks_total - last_used_block - 1) as u16).to_le_bytes());
    bgdt[14..16].copy_from_slice(&((inodes_total - 6) as u16).to_le_bytes());
    bgdt[16..18].copy_from_slice(&3u16.to_le_bytes());

    for b in 0..=last_used_block {
        let byte_idx = (b / 8) as usize;
        let bit_idx = b % 8;
        img[3072 + byte_idx] |= 1 << bit_idx;
    }
    for &ino in &[1u32, 2, 11, 12, 13, 14] {
        let byte_idx = (ino / 8) as usize;
        let bit_idx = ino % 8;
        img[4096 + byte_idx] |= 1 << bit_idx;
    }

    fn write_inode(table: &mut [u8], inum: u32, mode: u16, size: u32, block0: u32, links: u16) {
        let offset = ((inum - 1) * 128) as usize;
        table[offset..offset + 2].copy_from_slice(&mode.to_le_bytes());
        table[offset + 4..offset + 8].copy_from_slice(&size.to_le_bytes());
        table[offset + 26..offset + 28].copy_from_slice(&links.to_le_bytes());
        table[offset + 28..offset + 32].copy_from_slice(&((size + 511) / 512 * 2).to_le_bytes());
        table[offset + 40..offset + 44].copy_from_slice(&block0.to_le_bytes());
    }

    let inode_table_start = 5120usize;
    let it = &mut img[inode_table_start..inode_table_start + (inodes_total as usize * inode_size as usize)];
    write_inode(it, 2, 0x41ED, block_size, root_data_block, 3);
    write_inode(it, 12, 0x41ED, block_size, subdir_data_block, 2);
    write_inode(it, 13, 0x81A4, 6, file_data_block, 1);
    write_inode(it, 14, 0x81A4, 5, subdir_file_block, 1);

    fn write_dirent(data: &mut [u8], offset: usize, inode: u32, rec_len: u16, name_len: u8, ftype: u8, name: &[u8]) {
        data[offset..offset + 4].copy_from_slice(&inode.to_le_bytes());
        data[offset + 4..offset + 6].copy_from_slice(&rec_len.to_le_bytes());
        data[offset + 6] = name_len;
        data[offset + 7] = ftype;
        data[offset + 8..offset + 8 + name.len()].copy_from_slice(name);
    }

    let root_data = &mut img[(root_data_block * block_size) as usize..][..block_size as usize];
    write_dirent(root_data, 0, 2, 12, 1, 2, b".");
    write_dirent(root_data, 12, 2, 12, 2, 2, b"..");
    write_dirent(root_data, 24, 11, 20, 10, 2, b"lost+found");
    write_dirent(root_data, 44, 12, 16, 6, 2, b"subdir");
    write_dirent(root_data, 60, 13, 964, 5, 1, b"a.txt");

    let subdir_data = &mut img[(subdir_data_block * block_size) as usize..][..block_size as usize];
    write_dirent(subdir_data, 0, 12, 12, 1, 2, b".");
    write_dirent(subdir_data, 12, 2, 12, 2, 2, b"..");
    write_dirent(subdir_data, 24, 14, 1000, 5, 1, b"b.txt");

    let file_data = &mut img[(file_data_block * block_size) as usize..][..block_size as usize];
    file_data[..6].copy_from_slice(b"Hello\n");

    let subdir_file = &mut img[(subdir_file_block * block_size) as usize..][..block_size as usize];
    subdir_file[..5].copy_from_slice(b"Data\n");

    std::fs::write(path, img)
}

const FTP_PORT: u16 = 2121;

fn read_ftp_response(stream: &mut std::net::TcpStream) -> String {
    let mut buf = [0u8; 4096];
    let mut resp = String::new();
    loop {
        let n = stream.read(&mut buf).expect("read ftp response");
        if n == 0 { break; }
        resp.push_str(&String::from_utf8_lossy(&buf[..n]));
        if resp.ends_with("\r\n") {
            break;
        }
    }
    resp
}

#[test]
#[cfg(target_os = "linux")]
fn test_sys_lab3_ftp_nasm() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let img_path = tmp.path().join("test.ext3");
    create_test_ext3(img_path.to_str().unwrap()).expect("create ext3 image");

    // Use the pre-built compiler binary
    let compiler = std::path::PathBuf::from("target/release/mylang-parser");
    assert!(compiler.exists(), "compiler not built at {compiler:?}");

    let out_dir = tmp.path().join("out");
    let src = "labs-examples/system-programms/lab-3/input.mylang";
    let status = std::process::Command::new(&compiler)
        .args([src, "-o", out_dir.to_str().unwrap(), "-t", "nasm", "--os", "linux"])
        .status()
        .expect("compiler failed to start");
    assert!(status.success(), "compilation failed");

    // Start the FTP server binary
    let exe = out_dir.join(exe_name());
    let mut child = std::process::Command::new(&exe)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .current_dir(tmp.path())
        .spawn()
        .expect("spawn");

    // Send the ext3 image path to stdin
use std::io::{Read, Write};
    child.stdin.take().unwrap().write_all(format!("{}\n", img_path.display()).as_bytes()).ok();

    // Connect to the FTP server with retries
    use std::net::TcpStream;
    use std::time::{Duration, Instant};
    let start = Instant::now();
    let timeout = Duration::from_secs(8);
    let mut stream = loop {
        if let Ok(s) = TcpStream::connect(format!("127.0.0.1:{FTP_PORT}")) {
            break s;
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            panic!("timed out connecting to FTP server on port {FTP_PORT}");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

    // Read 220 greeting
    let resp = read_ftp_response(&mut stream);
    assert!(resp.contains("220"), "expected 220 greeting, got: {resp:?}");

    // Helper to send command and read response
    let mut cmd = |c: &[u8]| -> String {
        stream.write_all(c).ok();
        read_ftp_response(&mut stream)
    };

    // USER
    let r = cmd(b"USER ftp\r\n");
    assert!(r.contains("331"), "USER should get 331, got: {r:?}");

    // PASS
    let r = cmd(b"PASS test\r\n");
    assert!(r.contains("230"), "PASS should get 230, got: {r:?}");

    // SYST
    let r = cmd(b"SYST\r\n");
    assert!(r.contains("215"), "SYST should get 215, got: {r:?}");

    // TYPE
    let r = cmd(b"TYPE I\r\n");
    assert!(r.contains("200"), "TYPE should get 200, got: {r:?}");

    // PWD
    let r = cmd(b"PWD\r\n");
    assert!(r.contains("257"), "PWD should get 257, got: {r:?}");
    assert!(r.contains("/"), "PWD should contain root path");

    // PASV
    let r = cmd(b"PASV\r\n");
    assert!(r.contains("227"), "PASV should get 227, got: {r:?}");

    // Parse PASV response: "227 Entering Passive Mode (h1,h2,h3,h4,p1,p2)"
    let pasv = r.clone();
    let paren_start = pasv.find('(').unwrap();
    let paren_end = pasv.find(')').unwrap();
    let nums: Vec<u16> = pasv[paren_start + 1..paren_end]
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    assert_eq!(nums.len(), 6, "PASV should have 6 numbers");
    let data_port = nums[4] * 256 + nums[5];

    // Open data connection
    let mut data = TcpStream::connect(format!("127.0.0.1:{data_port}"))
        .expect("connect data port");
    data.set_read_timeout(Some(Duration::from_secs(5))).ok();

    // LIST
    let r = cmd(b"LIST\r\n");
    assert!(r.contains("150"), "LIST should get 150, got: {r:?}");
    assert!(r.contains("226"), "LIST should complete with 226, got: {r:?}");

    // Read directory listing from data connection
    let mut list_buf = Vec::new();
    data.read_to_end(&mut list_buf).ok();
    let listing = String::from_utf8_lossy(&list_buf);
    assert!(listing.contains("a.txt"), "LIST should contain a.txt:\n{listing}");
    assert!(listing.contains("subdir"), "LIST should contain subdir:\n{listing}");

    // --- RETR a.txt ---
    // PASV again
    let r = cmd(b"PASV\r\n");
    assert!(r.contains("227"), "PASV should get 227");

    let paren_start = r.find('(').unwrap();
    let paren_end = r.find(')').unwrap();
    let nums: Vec<u16> = r[paren_start + 1..paren_end]
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    let data_port = nums[4] * 256 + nums[5];

    let mut data = TcpStream::connect(format!("127.0.0.1:{data_port}"))
        .expect("connect data port");
    data.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let r = cmd(b"RETR a.txt\r\n");
    assert!(r.contains("150"), "RETR should get 150, got: {r:?}");
    assert!(r.contains("226"), "RETR should complete with 226, got: {r:?}");

    let mut file_buf = Vec::new();
    data.read_to_end(&mut file_buf).ok();
    assert_eq!(String::from_utf8_lossy(&file_buf), "Hello\n");

    // --- CWD subdir ---
    let r = cmd(b"CWD subdir\r\n");
    assert!(r.contains("250"), "CWD should get 250, got: {r:?}");

    let r = cmd(b"PWD\r\n");
    assert!(r.contains("/subdir"), "PWD should show /subdir, got: {r:?}");

    // --- RETR b.txt from subdir ---
    let r = cmd(b"PASV\r\n");
    assert!(r.contains("227"));

    let paren_start = r.find('(').unwrap();
    let paren_end = r.find(')').unwrap();
    let nums: Vec<u16> = r[paren_start + 1..paren_end]
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    let data_port = nums[4] * 256 + nums[5];

    let mut data = TcpStream::connect(format!("127.0.0.1:{data_port}"))
        .expect("connect data port");
    data.set_read_timeout(Some(Duration::from_secs(5))).ok();

    let r = cmd(b"RETR b.txt\r\n");
    assert!(r.contains("150"), "RETR b.txt should get 150, got: {r:?}");
    assert!(r.contains("226"), "RETR b.txt should complete, got: {r:?}");

    let mut file_buf = Vec::new();
    data.read_to_end(&mut file_buf).ok();
    assert_eq!(String::from_utf8_lossy(&file_buf), "Data\n");

    // --- CWD .. ---
    let r = cmd(b"CWD ..\r\n");
    assert!(r.contains("250"));

    let r = cmd(b"PWD\r\n");
    assert!(r.contains("/"), "PWD should show / after CWD ..");

    // --- NOOP ---
    let r = cmd(b"NOOP\r\n");
    assert!(r.contains("200"), "NOOP should get 200");

    // --- QUIT ---
    let r = cmd(b"QUIT\r\n");
    assert!(r.contains("221"), "QUIT should get 221");

    // Wait for the binary to exit cleanly
    let deadline = Instant::now() + Duration::from_secs(5);
    let exited = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                assert!(status.success(), "server should exit with 0, got {status}");
                break true;
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    break false;
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                child.kill().ok();
                panic!("child wait error: {e}");
            }
        }
    };
    assert!(exited, "server did not exit after QUIT");
}

// ========== sys-lab-2: Coroutine Map-Reduce Pipeline ==========

#[test]
#[cfg(target_os = "linux")]
fn test_sys_lab2_pipeline_nasm() {
    let out = compile_sys_file("lab-2/input.mylang").expect("compile/run failed");

    // All 7 queries must complete
    assert!(out.contains("Done"), "should complete:\n{out}");

    // Q1: INNER JOIN
    assert!(out.contains("Q1"), "should run Q1:\n{out}");
    assert!(out.contains("DiffPass"), "Q1 should print type name:\n{out}");
    assert!(out.contains("Found: 4"), "Q1 should find 4:\n{out}");

    // Q2: LEFT JOIN
    assert!(out.contains("Q2"), "should run Q2:\n{out}");
    assert!(out.contains("163276, OK500"), "Q2 should contain OK500:\n{out}");

    // Q3: COUNT
    assert!(out.contains("Q3"), "should run Q3:\n{out}");
    assert!(out.contains("Count: 6"), "Q3 should find 6:\n{out}");

    // Q4: Plans >2 groups on CE
    assert!(out.contains("Q4"), "should run Q4:\n{out}");
    assert!(out.contains("101: 3 groups"), "Q4 should find 101:\n{out}");
    assert!(out.contains("104: 3 groups"), "Q4 should find 104:\n{out}");

    // Q5: Avg grades
    assert!(out.contains("Q5"), "should run Q5:\n{out}");
    assert!(out.contains("Zaitsev"), "Q5 should contain Zaitsev:\n{out}");
    assert!(out.contains("Grigoriev"), "Q5 should contain Grigoriev:\n{out}");

    // Q6: After 2012-09-01 – 4 students
    assert!(out.contains("Q6"), "should run Q6:\n{out}");
    assert!(out.contains("Timofeev"), "Q6 should contain Timofeev:\n{out}");
    assert!(out.contains("Count: 4"), "Q6 should find 4:\n{out}");

    // Q7: Same surname – 12 rows
    assert!(out.contains("Q7"), "should run Q7:\n{out}");
    assert!(out.contains("Groups: 12"), "Q7 should have 12:\n{out}");

    assert!(out.len() > 400, "should produce output (got {} bytes)", out.len());
}
