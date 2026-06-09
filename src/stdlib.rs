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
            ("Sleep", ("ms: int", "")), // Windows Sleep (milliseconds) - blocking
        ];
        decls.into_iter().find(|(n, _)| *n == name).map(|(_, sig)| sig)
    }
}
