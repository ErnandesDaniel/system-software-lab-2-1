use std::fs;
use std::path::Path;

use crate::codegen::nasm::AsmGenerator;
use crate::driver::CompilerDriver;
use crate::ir::IrProgram;
use crate::OsTarget;

impl CompilerDriver {
    pub fn generate_nasm(ir: &IrProgram, output_dir: &str, os: OsTarget) {
        use std::process::Command;

        let (format, obj_ext, exe_name) = match os {
            OsTarget::Windows => ("win64", "obj", "program.exe"),
            OsTarget::Linux => ("elf64", "o", "program"),
        };

        let global_names: Vec<String> = ir.globals.iter().map(|g| g.name.clone()).collect();
        for func in &ir.functions {
            let mut gen = AsmGenerator::with_os(os);
            gen.set_global_names(&global_names);
            let mut asm = gen.generate_single_function(func);
            if !ir.globals.is_empty() {
                let mut externs = String::new();
                for g in &ir.globals {
                    externs.push_str(&format!("extern {}\n", g.name));
                }
                asm.insert_str(0, &externs);
            }
            let path = Path::new(output_dir).join(format!("{}.asm", func.name));

            if let Err(e) = fs::write(&path, &asm) {
                eprintln!("Failed to write assembly: {e}");
            }
        }

        if !ir.globals.is_empty() {
            let mut globals_asm = String::from("bits 64\ndefault rel\nsection .data\n");
            globals_asm.push_str(&AsmGenerator::generate_globals_asm(&ir.globals));
            let path = Path::new(output_dir).join("globals.asm");
            let _ = fs::write(&path, &globals_asm);
        }

        let mut obj_files = Vec::new();

        for func in &ir.functions {
            let asm_path = Path::new(output_dir).join(format!("{}.asm", func.name));
            let obj_path = Path::new(output_dir).join(format!("{}.{}", func.name, obj_ext));

            let output = Command::new("nasm")
                .args([
                    "-f",
                    format,
                    "-O0",
                    "-o",
                    obj_path.to_string_lossy().as_ref(),
                    asm_path.to_string_lossy().as_ref(),
                ])
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        obj_files.push(obj_path);
                    } else {
                        eprintln!("NASM failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to run NASM: {e}"),
            }
        }

        let globals_asm = Path::new(output_dir).join("globals.asm");
        if globals_asm.exists() {
            let globals_obj = Path::new(output_dir).join(format!("globals.{}", obj_ext));
            let output = Command::new("nasm")
                .args([
                    "-f",
                    format,
                    "-O0",
                    "-o",
                    globals_obj.to_string_lossy().as_ref(),
                    globals_asm.to_string_lossy().as_ref(),
                ])
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    obj_files.push(globals_obj);
                }
            }
        }

        // Compile NASM runtime files
        struct RuntimeFile<'a> {
            name: &'a str,
            lang: &'a str,
            enabled: bool,
        }
        let runtimes: [RuntimeFile; 2] = [
            RuntimeFile {
                name: "coro_linux.c",
                lang: "c",
                enabled: os == OsTarget::Linux,
            },
            RuntimeFile {
                name: "context.asm",
                lang: "asm",
                enabled: os == OsTarget::Linux,
            },
        ];
        for rf in &runtimes {
            if !rf.enabled {
                continue;
            }
            let src = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src/codegen/nasm/runtime")
                .join(rf.name);
            let obj_name = format!("{}.{}", rf.name.replace(".asm", "").replace(".c", ""), obj_ext);
            let obj_path = Path::new(output_dir).join(&obj_name);
            let result = if rf.lang == "c" {
                Command::new("gcc")
                    .args(["-c", "-o"])
                    .arg(obj_path.to_string_lossy().as_ref())
                    .arg(src.to_string_lossy().as_ref())
                    .output()
            } else {
                Command::new("nasm")
                    .args(["-f", format, "-O0", "-o"])
                    .arg(obj_path.to_string_lossy().as_ref())
                    .arg(src.to_string_lossy().as_ref())
                    .output()
            };
            match result {
                Ok(out) => {
                    if out.status.success() {
                        obj_files.push(obj_path);
                    } else {
                        eprintln!(
                            "{} ({}) failed: {}",
                            rf.lang,
                            rf.name,
                            String::from_utf8_lossy(&out.stderr)
                        );
                    }
                }
                Err(e) => eprintln!("Failed to run {} on {}: {e}", rf.lang, rf.name),
            }
        }

        if !obj_files.is_empty() {
            let exe_path = Path::new(output_dir).join(exe_name);
            let mut args: Vec<String> = obj_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
            if os == OsTarget::Windows {
                args.push("-Wl,/subsystem:console".to_string());
            }
            args.push("-o".to_string());
            args.push(exe_path.to_string_lossy().to_string());

            let result = if os == OsTarget::Linux {
                let mut linux_args = vec!["-no-pie".to_string()];
                linux_args.extend(args.iter().cloned());
                Command::new("gcc").args(&linux_args).output()
            } else {
                let link = |linker: &str| Command::new(linker).args(&args).output();
                link("clang").or_else(|_| link("gcc")).or_else(|_| link("mingw32-gcc"))
            };
            match result {
                Ok(out) => {
                    if !out.status.success() {
                        eprintln!("Link failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to run linker: {e}"),
            }
        }
    }
}
