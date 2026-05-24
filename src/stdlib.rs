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
        funcs.insert("Sleep");  // Windows Sleep (ms) - blocking

        // Process functions
        funcs.insert("exit");
        funcs.insert("system");

        // EntityStore functions (JVM daemon)
        funcs.insert("map_put");
        funcs.insert("map_get");
        funcs.insert("map_remove");
        funcs.insert("map_has");
        funcs.insert("map_size");
        funcs.insert("map_key");
        funcs.insert("map_list");

        // SHM functions (JVM daemon)
        funcs.insert("shm_read_state");
        funcs.insert("shm_read_byte");
        funcs.insert("shm_read_str");
        funcs.insert("shm_write_state");
        funcs.insert("shm_write_resp");
        funcs.insert("shm_wait_event");
        funcs.insert("shm_find_null");

        // Coroutine runtime
        funcs.insert("resume_coroutine");
        funcs.insert("create_coroutine");
        funcs.insert("coro_init");
        funcs.insert("run_scheduler");

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
            ("clock", ("", "int")),
            ("Sleep", ("ms: int", "int")),  // Windows Sleep (milliseconds) - blocking
            ("putchar", ("c: int", "int")),
            // SHM functions (JVM daemon)
            ("shm_read_state", ("", "int")),
            ("shm_read_byte", ("pos: int", "int")),
            ("shm_read_str", ("pos: int", "string")),
            ("shm_write_state", ("state: int", "")),
            ("shm_write_resp", ("result: int, payload: string", "")),
            ("shm_wait_event", ("", "")),
            ("shm_find_null", ("start: int", "int")),
            // Coroutine runtime
            ("resume_coroutine", ("index: int", "int")),
            ("create_coroutine", ("", "")),
        ];
        decls
            .into_iter()
            .find(|(n, _)| *n == name)
            .map(|(_, sig)| sig)
    }

    #[allow(dead_code)]
    pub fn generate_extern_decls() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            // (name, params, return_type)
            ("getchar", "", "int"),
            ("putchar", "c: int", "int"),
            ("puts", "s: string", "int"),
            ("printf", "format: string, value: int", "int"),
            ("scanf", "format: string", "int"),
            ("strlen", "s: string", "int"),
            ("strcpy", "dest: string, src: string", "string"),
            ("strcat", "dest: string, src: string", "string"),
            ("strcmp", "s1: string, s2: string", "int"),
            ("malloc", "size: int", "string"),
            ("free", "ptr: string", ""),
            ("abs", "n: int", "int"),
            ("rand", "", "int"),
            ("srand", "seed: int", ""),
            ("time", "dummy: int", "int"),
            ("exit", "code: int", ""),
        ]
    }
}
