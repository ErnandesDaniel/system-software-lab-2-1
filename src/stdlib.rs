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
        funcs.insert("usleep");

        // Process functions
        funcs.insert("exit");
        funcs.insert("system");

        funcs
    }

    pub fn is_stdlib(name: &str) -> bool {
        Self::all().contains(name)
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
