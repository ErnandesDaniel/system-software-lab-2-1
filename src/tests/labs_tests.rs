/// Integration tests for VM labs.
/// Runs the exact commands from each lab's README.md,
/// then verifies expected output.
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

static CARGO_LOCK: Mutex<()> = Mutex::new(());

fn exe_name() -> &'static str {
    if cfg!(target_os = "linux") { "program" } else { "program.exe" }
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

// ========== sys-lab-3: Ext3 FTP-like CLI ==========

#[test]
#[cfg(target_os = "linux")]
fn test_sys_lab3_ext3_nasm() {
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

        // Block 1: Superblock at byte offset 1024
        let sb = &mut img[1024..2048];
        sb[0..4].copy_from_slice(&inodes_total.to_le_bytes());
        sb[4..8].copy_from_slice(&blocks_total.to_le_bytes());
        sb[20..24].copy_from_slice(&1u32.to_le_bytes());
        sb[24..28].copy_from_slice(&0u32.to_le_bytes());
        sb[32..36].copy_from_slice(&blocks_total.to_le_bytes());
        sb[40..44].copy_from_slice(&inodes_total.to_le_bytes());
        sb[56..58].copy_from_slice(&0xEF53u16.to_le_bytes());
        sb[88..90].copy_from_slice(&(inode_size as u16).to_le_bytes());

        // Block 2: BGDT
        let bgdt = &mut img[2048..2080];
        bgdt[0..4].copy_from_slice(&bgdt_block_bitmap.to_le_bytes());
        bgdt[4..8].copy_from_slice(&bgdt_inode_bitmap.to_le_bytes());
        bgdt[8..12].copy_from_slice(&bgdt_inode_table.to_le_bytes());
        bgdt[12..14].copy_from_slice(&((blocks_total - last_used_block - 1) as u16).to_le_bytes());
        // 6 used inodes: 1,2,11,12,13,14
        bgdt[14..16].copy_from_slice(&((inodes_total - 6) as u16).to_le_bytes());
        bgdt[16..18].copy_from_slice(&3u16.to_le_bytes());

        // Block 3: Block bitmap
        for b in 0..=last_used_block {
            let byte_idx = (b / 8) as usize;
            let bit_idx = b % 8;
            img[3072 + byte_idx] |= 1 << bit_idx;
        }

        // Block 4: Inode bitmap — inodes 1,2,11,12,13,14 used
        for &ino in &[1u32, 2, 11, 12, 13, 14] {
            let byte_idx = (ino / 8) as usize;
            let bit_idx = ino % 8;
            img[4096 + byte_idx] |= 1 << bit_idx;
        }

        // Blocks 5-8: Inode table
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

        // Block 9: Root directory data
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
        write_dirent(root_data, 24, 11, 12, 10, 2, b"lost+found");
        write_dirent(root_data, 36, 12, 12, 6, 2, b"subdir");
        write_dirent(root_data, 48, 13, 976, 5, 1, b"a.txt");

        // Block 10: subdir — now includes b.txt
        let subdir_data = &mut img[(subdir_data_block * block_size) as usize..][..block_size as usize];
        write_dirent(subdir_data, 0, 12, 12, 1, 2, b".");
        write_dirent(subdir_data, 12, 2, 12, 2, 2, b"..");
        write_dirent(subdir_data, 24, 14, 1000, 5, 1, b"b.txt");

        // Block 11: a.txt content
        let file_data = &mut img[(file_data_block * block_size) as usize..][..block_size as usize];
        file_data[..6].copy_from_slice(b"Hello\n");

        // Block 12: b.txt content (inside subdir)
        let subdir_file = &mut img[(subdir_file_block * block_size) as usize..][..block_size as usize];
        subdir_file[..5].copy_from_slice(b"Data\n");

        std::fs::write(path, img)
    }
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let img_path = tmp.path().join("test.ext3");
    create_test_ext3(img_path.to_str().unwrap()).expect("create ext3 image");

    let out_dir = tmp.path().join("out");
    let src = "labs-examples/system-programms/lab-3/input.mylang";
    let _lock = CARGO_LOCK.lock().unwrap();
    assert!(
        cargo(&[src, "-o", out_dir.to_str().unwrap(), "-t", "nasm"]),
        "compile failed"
    );
    drop(_lock);

    let input = format!(
        "{}\nPWD\nLIST\nRETR a.txt\nCWD subdir\nPWD\nLIST\nRETR b.txt\nQUIT\n",
        img_path.to_str().unwrap()
    );

    let exe = out_dir.join(exe_name());
    let mut child = std::process::Command::new(&exe)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn");

    use std::io::Write;
    child.stdin.take().unwrap().write_all(input.as_bytes()).ok();
    let output = child.wait_with_output().expect("wait");
    let stdout = clean(&String::from_utf8_lossy(&output.stdout));

    assert!(stdout.contains("Ext3 filesystem detected"), "should detect ext3:\n{stdout}");

    // PWD shows root
    assert!(stdout.contains("/"), "should show root path:\n{stdout}");

    // LIST shows files in root
    assert!(stdout.contains("subdir"), "should list subdir:\n{stdout}");
    assert!(stdout.contains("a.txt"), "should list a.txt:\n{stdout}");

    // RETR a.txt from root
    assert!(stdout.contains("226 Transfer complete"), "RETR should complete:\n{stdout}");

    // CWD actually changed directory — PWD shows /subdir
    assert!(stdout.contains("/subdir"), "should change to subdir:\n{stdout}");

    // LIST after CWD shows b.txt (file inside subdir)
    assert!(stdout.contains("b.txt"), "should list b.txt in subdir:\n{stdout}");

    // RETR b.txt from subdir
    assert!(stdout.contains("226 Transfer"), "second RETR should complete:\n{stdout}");
    assert!(stdout.contains("221 Goodbye"), "should quit cleanly:\n{stdout}");

    // Verify extracted a.txt content from root
    let extracted_root = tmp.path().join("a.txt");
    assert!(extracted_root.exists(), "a.txt should be extracted");
    let content_root = std::fs::read_to_string(&extracted_root).unwrap_or_default();
    assert_eq!(content_root, "Hello\n", "a.txt content mismatch");
    let _ = std::fs::remove_file(&extracted_root);

    // Verify extracted b.txt content from subdir
    let extracted_sub = tmp.path().join("b.txt");
    assert!(extracted_sub.exists(), "b.txt should be extracted");
    let content_sub = std::fs::read_to_string(&extracted_sub).unwrap_or_default();
    assert_eq!(content_sub, "Data\n", "b.txt content mismatch");
    let _ = std::fs::remove_file(&extracted_sub);
}

// ========== sys-lab-2: Coroutine Map-Reduce Pipeline ==========

#[test]
#[cfg(target_os = "linux")]
fn test_sys_lab2_pipeline_nasm() {
    let out = compile_sys_file("lab-2/input.mylang").expect("compile/run failed");

    // All 7 queries must complete
    assert!(out.contains("Done"), "should complete:\n{out}");

    // Q1: INNER JOIN vedomosti(tid=3,pid>153285) × types
    // Expect: "Colloquium, 153286" ... "Found: 4"
    assert!(out.contains("Q1"), "should run Q1:\n{out}");
    assert!(out.contains("Colloquium"), "Q1 should print type name:\n{out}");
    assert!(out.contains("Found:"), "should have results:\n{out}");

    // Q2: LEFT JOIN studies(pid=163276) × students(start=2008-09-01)
    assert!(out.contains("Q2"), "should run Q2:\n{out}");
    assert!(out.contains("163276"), "Q2 should contain pid 163276:\n{out}");
    assert!(out.contains("OK500"), "Q2 should contain OK500:\n{out}");

    // Q3: COUNT without patronymic + FCE + student exists
    assert!(out.contains("Q3"), "should run Q3:\n{out}");
    assert!(out.contains("Count:"), "Q3 should have Count:\n{out}");

    // Q4: Plans >2 groups on Computer Engineering
    assert!(out.contains("Q4"), "should run Q4:\n{out}");

    // Q5: Avg grades 4100 >= 1100
    assert!(out.contains("Q5"), "should run Q5:\n{out}");

    // Q6: After 2012-09-01, course 1, part-time
    assert!(out.contains("Q6"), "should run Q6:\n{out}");

    // Q7: Same surname, diff bday
    assert!(out.contains("Q7"), "should run Q7:\n{out}");

    assert!(out.len() > 400, "should produce output (got {} bytes)", out.len());
}


