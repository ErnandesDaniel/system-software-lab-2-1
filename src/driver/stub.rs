use std::fs;
use std::path::Path;

use crate::driver::CompilerDriver;

fn desc_to_java_type(desc: &str) -> &str {
    match desc {
        "I" => "int",
        "Z" => "int",
        "[B" => "byte[]",
        _ => "int",
    }
}

impl CompilerDriver {
    pub fn generate_jvm_stub(output_dir: &str, global_info: &[(String, String, usize, usize)]) {
        let mut static_fields = String::new();
        let mut static_init = String::new();
        for (name, desc, outer_size, inner_size) in global_info {
            if *outer_size > 0 && *inner_size > 0 {
                static_fields.push_str(&format!("    static Object[] {} = new Object[{}];\n", name, outer_size));
                static_init.push_str(&format!(
                    "        for (int __i = 0; __i < {}; __i++) {}[__i] = new Object[{}];\n",
                    outer_size, name, inner_size
                ));
            } else if *outer_size > 0 {
                let jtype = desc_to_java_type(desc);
                static_fields.push_str(&format!("    static {}[] {};\n", jtype, name));
            } else {
                let jtype = desc_to_java_type(desc);
                static_fields.push_str(&format!("    static {} {};\n", jtype, name));
            }
        }

        let stub = format!(r#"import java.io.*;
import java.util.*;

public class RuntimeStub {{
    private static HashMap<String, String> store = new HashMap<>();
    private static Random random = new Random();

    // --- Global static fields ---
{static_fields}
    static {{
{static_init}    }}

    public static void main(String[] args) {{
        int result = Main.call();
        System.exit(result);
    }}

    // --- Standard IO ---

    public static int getchar() throws IOException {{
        return System.in.read();
    }}

    public static int putchar(int c) {{
        System.out.print((char) c);
        System.out.flush();
        return c;
    }}

    public static int puts(byte[] s) {{
        System.out.println(new String(s));
        System.out.flush();
        return s.length;
    }}

    public static byte[] malloc(int size) {{
        return new byte[size];
    }}

    public static void free(byte[] ptr) {{}}

    public static int printf(byte[] format, int value) {{
        System.out.print(new String(format)
                                .replace("%d", String.valueOf(value))
                                .replace("%c", String.valueOf((char) value))
                                .replace("%s", String.valueOf(value))
                                .replace("\\n", "\n")
                                .replace("\\t", "\t"));
        System.out.flush();
        return value;
    }}

    public static int rand() {{
        return random.nextInt();
    }}

    public static void srand(int seed) {{
        random = new Random(seed);
    }}

    public static int time(int dummy) {{
        return (int)(System.currentTimeMillis() / 1000);
    }}

    public static void Sleep(int ms) {{
        try {{ Thread.sleep(ms); }} catch (InterruptedException e) {{}}
    }}

    // --- Map functions (JVM) ---

    public static int map_put_jvm(byte[] name, byte[] value) {{
        synchronized (store) {{ store.put(new String(name), new String(value)); return 1; }}
    }}

    public static byte[] map_get_jvm(byte[] name) {{
        synchronized (store) {{ String v = store.get(new String(name)); return v != null ? v.getBytes() : new byte[0]; }}
    }}

    public static int map_has_jvm(byte[] name) {{
        synchronized (store) {{ return store.containsKey(new String(name)) ? 1 : 0; }}
    }}

    public static int map_remove_jvm(byte[] name) {{
        synchronized (store) {{ return store.remove(new String(name)) != null ? 1 : 0; }}
    }}

    public static int map_size_jvm() {{
        synchronized (store) {{ return store.size(); }}
    }}

    public static byte[] map_key_jvm(int i) {{
        synchronized (store) {{
            int idx = 0;
            for (String k : store.keySet()) {{
                if (idx == i) return k.getBytes();
                idx++;
            }}
            return new byte[0];
        }}
    }}

    public static byte[] map_list_jvm() {{
        synchronized (store) {{ return String.join(",", store.keySet()).getBytes(); }}
    }}
}}
"#,
            static_fields = static_fields,
            static_init = static_init,
        );
        let stub_path = Path::new(output_dir).join("RuntimeStub.java");
        if let Err(e) = fs::write(&stub_path, stub) {
            eprintln!("Failed to write RuntimeStub: {e}");
        }

        let runner = r#"import java.lang.reflect.Method;
import java.io.File;
import java.net.URL;
import java.net.URLClassLoader;

public class MainRunner {
    public static void main(String[] args) throws Exception {
        if (args.length < 1) {
            printUsage();
            System.exit(1);
        }

        String func = args[0];
        int[] params = new int[args.length - 1];
        for (int i = 1; i < args.length; i++) {
            params[i - 1] = Integer.parseInt(args[i]);
        }

        File currentDir = new File(".");
        URL[] urls = new URL[] { currentDir.toURI().toURL() };
        URLClassLoader classLoader = new URLClassLoader(urls, MainRunner.class.getClassLoader());

        String className = capitalizeFirst(func);
        Class<?> clazz;
        
        try {
            clazz = classLoader.loadClass(className);
        } catch (ClassNotFoundException e) {
            if (func.equals("main")) {
                clazz = classLoader.loadClass("Main");
            } else {
                System.err.println("Unknown function: " + func);
                System.err.println("Class not found: " + className);
                System.exit(1);
                return;
            }
        }

        Method targetMethod = null;
        for (Method method : clazz.getMethods()) {
            if (method.getName().equals("call") && method.getParameterCount() == params.length) {
                targetMethod = method;
                break;
            }
        }

        if (targetMethod == null) {
            System.err.println("Method 'call" + params.length + "' not found in class " + className);
            System.exit(1);
            return;
        }

        Object result = targetMethod.invoke(null, (Object[]) boxParams(params));
        System.out.println(result);
    }

    private static void printUsage() {
        System.out.println("Usage: java MainRunner <function> [args...]");
        System.out.println();
        System.out.println("Available functions:");
        System.out.println("  main             - main function");
        System.out.println("  factorial n      - n factorial");
        System.out.println("  power base exp   - base^exp");
        System.out.println("  sum a b          - a + b");
        System.out.println("  diff a b         - a - b");
        System.out.println("  product a b      - a * b");
    }

    private static String capitalizeFirst(String s) {
        if (s.isEmpty()) return s;
        return Character.toUpperCase(s.charAt(0)) + s.substring(1);
    }

    private static Integer[] boxParams(int[] params) {
        Integer[] result = new Integer[params.length];
        for (int i = 0; i < params.length; i++) {
            result[i] = params[i];
        }
        return result;
    }
}
"#;
        let runner_path = Path::new(output_dir).join("MainRunner.java");
        if let Err(e) = fs::write(&runner_path, runner) {
            eprintln!("Failed to write MainRunner: {e}");
        }
    }
}
