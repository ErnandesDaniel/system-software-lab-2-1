use std::collections::HashSet;

pub struct StdLib;

impl StdLib {
    pub fn all() -> HashSet<&'static str> {
        let mut funcs = HashSet::new();

        // Input/Output
        funcs.insert("getchar");
        funcs.insert("putchar");
        funcs.insert("puts");
        funcs.insert("printf");

        // String functions
        funcs.insert("strlen");
        funcs.insert("strcpy");
        funcs.insert("strcat");
        funcs.insert("strcmp");
        funcs.insert("strchr");

        // Memory functions
        funcs.insert("malloc");
        funcs.insert("free");
        funcs.insert("memcpy");

        // Math functions
        funcs.insert("rand");
        funcs.insert("srand");

        // Time functions
        funcs.insert("time");
        funcs.insert("sleep");
        funcs.insert("Sleep"); // Windows Sleep (ms) - blocking

        // File I/O functions
        funcs.insert("fopen");
        funcs.insert("fclose");
        funcs.insert("fgets");
        funcs.insert("fgetc");
        funcs.insert("feof");

        // Conversion functions
        funcs.insert("atoi");
        funcs.insert("sprintf");

        // Process functions
        funcs.insert("exit");
        funcs.insert("fflush");

        // Coroutine runtime functions
        funcs.insert("create_coroutine_nasm");
        funcs.insert("init_coroutine_runtime_nasm");
        funcs.insert("run_coroutine_runtime_nasm");
        funcs.insert("set_coroutine_scheduler_nasm");
        funcs.insert("get_current_coroutine_id_nasm");

        // Binary I/O functions (NASM-only wrappers)
        funcs.insert("fread_nasm");
        funcs.insert("fwrite_nasm");
        funcs.insert("fseek_nasm");
        funcs.insert("read_le32_nasm");
        funcs.insert("read_le16_nasm");
        funcs.insert("read_i8_nasm");

        // EntityStore functions (JVM daemon)
        funcs.insert("map_put_jvm");
        funcs.insert("map_get_jvm");
        funcs.insert("map_remove_jvm");
        funcs.insert("map_has_jvm");
        funcs.insert("map_size_jvm");
        funcs.insert("map_key_jvm");
        funcs.insert("map_list_jvm");

        funcs
    }

    pub fn is_stdlib(name: &str) -> bool {
        Self::all().contains(name)
    }

    pub fn get_signature(name: &str) -> Option<(&'static str, &'static str)> {
        let decls = vec![
            ("getchar", ("", "int")),
            ("putchar", ("c: int", "int")),
            ("puts", ("s: string", "int")),
            ("printf", ("format: string, value: int", "int")),
            ("strlen", ("s: string", "int")),
            ("strcpy", ("dest: string, src: string", "string")),
            ("strcat", ("dest: string, src: string", "string")),
            ("strcmp", ("s1: string, s2: string", "int")),
            ("malloc", ("size: int", "string")),
            ("free", ("ptr: string", "")),
            ("rand", ("", "int")),
            ("srand", ("seed: int", "")),
            ("time", ("dummy: int", "int")),
            ("exit", ("code: int", "")),
            ("fflush", ("dummy: int", "int")),
            ("memcpy", ("dest: string, src: string, n: int", "string")),
            ("strchr", ("s: string, c: int", "string")),
            ("fopen", ("filename: string, mode: string", "string")),
            ("fclose", ("file: string", "int")),
            ("fgets", ("buf: string, maxcount: int, file: string", "string")),
            ("fgetc", ("file: string", "int")),
            ("feof", ("file: string", "int")),
            ("atoi", ("str: string", "int")),
            ("sprintf", ("buf: string, format: string, value: int", "int")),
            ("sleep", ("seconds: int", "int")),
            ("Sleep", ("ms: int", "")), // Windows Sleep (milliseconds) - blocking
            ("create_coroutine_nasm", ("fn: int", "int")),
            ("init_coroutine_runtime_nasm", ("", "")),
            ("run_coroutine_runtime_nasm", ("", "")),
            ("set_coroutine_scheduler_nasm", ("fn: int", "")),
            ("get_current_coroutine_id_nasm", ("", "int")),

            // Binary I/O (NASM wrappers)
            ("fread_nasm", ("buf: string, size: int, count: int, file: string", "int")),
            ("fwrite_nasm", ("buf: string, size: int, count: int, file: string", "int")),
            ("fseek_nasm", ("file: string, offset: int, whence: int", "int")),
            ("read_le32_nasm", ("buf: string, offset: int", "int")),
            ("read_le16_nasm", ("buf: string, offset: int", "int")),
            ("read_i8_nasm", ("buf: string, offset: int", "int")),
        ];
        decls.into_iter().find(|(n, _)| *n == name).map(|(_, sig)| sig)
    }
}
