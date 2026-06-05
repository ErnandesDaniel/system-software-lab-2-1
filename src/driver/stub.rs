use std::fs;
use std::path::Path;

use crate::driver::CompilerDriver;

fn jvm_desc_to_java_type(desc: &str) -> String {
    let dims = desc.chars().filter(|&c| c == '[').count();
    let base = &desc[dims..];
    let base_type = match base {
        "B" => "byte",
        "C" => "char",
        "S" => "short",
        "J" => "long",
        "F" => "float",
        "D" => "double",
        "Ljava/lang/Object;" => "Object",
        _ => "int",
    };
    if dims > 0 {
        format!("{}{}", base_type, "[]".repeat(dims))
    } else {
        base_type.to_string()
    }
}

/// Build Java field declaration + initializer for a global with JVM descriptor.
fn jvm_field_decl_init(name: &str, desc: &str, outer_size: usize, inner_size: usize) -> (String, String) {
    if outer_size > 0 && inner_size > 0 {
        // 2D object array (struct-backed)
        let decl = format!("    static Object[] {} = new Object[{}];", name, outer_size);
        let init = format!(
            "        for (int __i = 0; __i < {}; __i++) {}[__i] = new Object[{}];\n",
            outer_size, name, inner_size
        );
        return (decl, init);
    }

    let full_type = jvm_desc_to_java_type(desc);
    let dims = desc.chars().filter(|&c| c == '[').count();

    if outer_size > 0 {
        // N-D array — declare & allocate outer dimension
        let decl = format!("    static {} {};", full_type, name);
        // Strip one dimension from the base type for the allocation
        let alloc = if dims <= 1 {
            format!("new {}[{}]", full_type.trim_end_matches("[]"), outer_size)
        } else {
            // For multi-dimensional, keep inner dimensions as trailing brackets
            // e.g., byte[][] -> new byte[outer][]
            let inner_suffix = "[]".repeat(dims - 1);
            let base = full_type.trim_end_matches("[]");
            format!("new {}[{}]{}", base, outer_size, inner_suffix)
        };
        let init = format!("        {} = {};\n", name, alloc);
        (decl, init)
    } else {
        // Scalar — just declare, no init (handled by scalar_inits)
        let decl = format!("    static {} {};", full_type, name);
        (decl, String::new())
    }
}

impl CompilerDriver {
    pub fn generate_jvm_stub(output_dir: &str, global_info: &[(String, String, usize, usize)], scalar_inits: &str) {
        let mut static_fields = String::new();
        let mut static_init = String::new();
        for (name, desc, outer_size, inner_size) in global_info {
            let (decl, init) = jvm_field_decl_init(name, desc, *outer_size, *inner_size);
            static_fields.push_str(&decl);
            static_fields.push('\n');
            if !init.is_empty() {
                static_init.push_str(&init);
            }
        }

        let stub = format!(
            r#"import java.io.*;
import java.util.*;
import java.nio.charset.StandardCharsets;

public class RuntimeStub {{
    private static HashMap<String, String> store = new HashMap<>();
    private static Random random = new Random();
    private static ArrayList<byte[]> gc_data = new ArrayList<>();

    // --- Global static fields ---
{static_fields}
    static {{
{scalar_inits}{static_init}    }}

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
        System.out.println(new String(s, StandardCharsets.UTF_8));
        System.out.flush();
        return s.length;
    }}

    public static byte[] malloc(int size) {{
        byte[] mem = new byte[size];
        gc_data.add(mem);
        return mem;
    }}

    public static void free(byte[] ptr) {{
        if (ptr == null) return;
        gc_data.remove(ptr);
    }}

    public static int printf(byte[] format, int value) {{
        String fmt = new String(format, StandardCharsets.UTF_8);
        StringBuilder sb = new StringBuilder();
        for (int __i = 0; __i < fmt.length(); __i++) {{
            char c = fmt.charAt(__i);
            if (c != '%') {{ sb.append(c); continue; }}
            __i++;
            if (__i >= fmt.length()) {{ sb.append(c); break; }}
            if (fmt.charAt(__i) == 'd') {{ sb.append(value); continue; }}
            if (fmt.charAt(__i) == 'c') {{ sb.append((char) value); continue; }}
            if (fmt.charAt(__i) == 's') {{ sb.append(value); continue; }}
            boolean zeroPad = false;
            if (fmt.charAt(__i) == '0') {{ zeroPad = true; __i++; }}
            int width = 0;
            while (__i < fmt.length() && fmt.charAt(__i) >= '0' && fmt.charAt(__i) <= '9') {{
                width = width * 10 + (fmt.charAt(__i) - '0');
                __i++;
            }}
            if (__i < fmt.length() && (fmt.charAt(__i) == 'd' || fmt.charAt(__i) == 'c' || fmt.charAt(__i) == 's')) {{
                String num = String.valueOf(fmt.charAt(__i) == 'c' ? (char) value : value);
                while (num.length() < width) num = zeroPad ? "0" + num : " " + num;
                sb.append(num);
            }} else {{
                __i--;
                sb.append('%');
            }}
        }}
        System.out.print(sb.toString().replace("\\n", "\n").replace("\\t", "\t"));
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

    public static int fflush(int dummy) {{
        System.out.flush();
        return 0;
    }}

    // --- Map functions (JVM) ---

    public static int map_put_jvm(byte[] name, byte[] value) {{
        synchronized (store) {{ store.put(new String(name, StandardCharsets.UTF_8), new String(value, StandardCharsets.UTF_8)); return 1; }}
    }}

    public static byte[] map_get_jvm(byte[] name) {{
        synchronized (store) {{ String v = store.get(new String(name, StandardCharsets.UTF_8)); return v != null ? v.getBytes(StandardCharsets.UTF_8) : new byte[0]; }}
    }}

    public static int map_has_jvm(byte[] name) {{
        synchronized (store) {{ return store.containsKey(new String(name, StandardCharsets.UTF_8)) ? 1 : 0; }}
    }}

    public static int map_remove_jvm(byte[] name) {{
        synchronized (store) {{ return store.remove(new String(name, StandardCharsets.UTF_8)) != null ? 1 : 0; }}
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
            scalar_inits = scalar_inits,
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
