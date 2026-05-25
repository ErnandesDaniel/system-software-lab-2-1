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
        funcs.insert("scanf");

        // String functions
        funcs.insert("strlen");
        funcs.insert("strcpy");
        funcs.insert("strcat");
        funcs.insert("strcmp");
        funcs.insert("strchr");
        funcs.insert("strstr");

        // Memory functions
        funcs.insert("malloc");
        funcs.insert("free");
        funcs.insert("calloc");
        funcs.insert("realloc");
        funcs.insert("memcpy");
        funcs.insert("memset");
        funcs.insert("memcmp");

        // Math functions
        funcs.insert("abs");
        funcs.insert("rand");
        funcs.insert("srand");

        // Time functions
        funcs.insert("time");
        funcs.insert("clock");
        funcs.insert("sleep");
        funcs.insert("Sleep"); // Windows Sleep (ms) - blocking

        // File I/O functions
        funcs.insert("fopen");
        funcs.insert("fclose");
        funcs.insert("fgets");
        funcs.insert("fgetc");
        funcs.insert("feof");
        funcs.insert("fputs");

        // String byte access helpers (NASM)
        funcs.insert("str_get_byte");
        funcs.insert("str_set_byte");
        funcs.insert("str_offset");

        // Conversion functions
        funcs.insert("atoi");
        funcs.insert("sprintf");

        // Process functions
        funcs.insert("exit");
        funcs.insert("system");

        // EntityStore functions (JVM daemon)
        funcs.insert("map_put_jvm");
        funcs.insert("map_get_jvm");
        funcs.insert("map_remove_jvm");
        funcs.insert("map_has_jvm");
        funcs.insert("map_size_jvm");
        funcs.insert("map_key_jvm");
        funcs.insert("map_list_jvm");

        // SHM functions (JVM daemon)
        funcs.insert("shm_read_state_jvm");
        funcs.insert("shm_read_byte_jvm");
        funcs.insert("shm_read_str_jvm");
        funcs.insert("shm_write_state_jvm");
        funcs.insert("shm_write_resp_jvm");
        funcs.insert("shm_wait_event_jvm");
        funcs.insert("shm_find_null_jvm");

        // Coroutine runtime (NASM) — ручное планирование
        funcs.insert("resume_coroutine_nasm");
        funcs.insert("create_coroutine_nasm");
        funcs.insert("coro_init_nasm");

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
            ("scanf", ("format: string", "int")),
            ("strlen", ("s: string", "int")),
            ("strcpy", ("dest: string, src: string", "string")),
            ("strcat", ("dest: string, src: string", "string")),
            ("strcmp", ("s1: string, s2: string", "int")),
            ("malloc", ("size: int", "string")),
            ("free", ("ptr: string", "")),
            ("abs", ("n: int", "int")),
            ("rand", ("", "int")),
            ("srand", ("seed: int", "")),
            ("time", ("dummy: int", "int")),
            ("exit", ("code: int", "")),
            ("calloc", ("nmemb: int, size: int", "string")),
            ("realloc", ("ptr: string, size: int", "string")),
            ("memcpy", ("dest: string, src: string, n: int", "string")),
            ("memset", ("s: string, c: int, n: int", "string")),
            ("memcmp", ("s1: string, s2: string, n: int", "int")),
            ("strchr", ("s: string, c: int", "string")),
            ("strstr", ("haystack: string, needle: string", "string")),
            ("system", ("command: string", "int")),
            ("fopen", ("filename: string, mode: string", "string")),
            ("fclose", ("file: string", "int")),
            ("fgets", ("buf: string, maxcount: int, file: string", "string")),
            ("fgetc", ("file: string", "int")),
            ("feof", ("file: string", "int")),
            ("fputs", ("str: string, file: string", "int")),
            ("str_get_byte", ("str: string, idx: int", "int")),
            ("str_set_byte", ("str: string, idx: int, val: int", "int")),
            ("str_offset", ("str: string, idx: int", "string")),
            ("atoi", ("str: string", "int")),
            ("sprintf", ("buf: string, format: string, value: int", "int")),
            ("clock", ("", "int")),
            ("Sleep", ("ms: int", "int")), // Windows Sleep (milliseconds) - blocking
            // SHM functions (JVM daemon)
            ("shm_read_state_jvm", ("", "int")),
            ("shm_read_byte_jvm", ("pos: int", "int")),
            ("shm_read_str_jvm", ("pos: int", "string")),
            ("shm_write_state_jvm", ("state: int", "")),
            ("shm_write_resp_jvm", ("result: int, payload: string", "")),
            ("shm_wait_event_jvm", ("", "")),
            ("shm_find_null_jvm", ("start: int", "int")),
            // Coroutine runtime (NASM)
            ("resume_coroutine_nasm", ("index: int", "int")),
            ("create_coroutine_nasm", ("", "")),
        ];
        decls.into_iter().find(|(n, _)| *n == name).map(|(_, sig)| sig)
    }
}
