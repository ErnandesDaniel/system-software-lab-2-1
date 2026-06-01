use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::IrType;

impl JvmGenerator {
    /// Generate RuntimeStub.java source with global static fields and utility methods.
    pub fn runtime_stub_source(&self) -> String {
        let mut static_fields = String::new();
        let mut static_init = String::new();
        for (name, ir_type) in &self.global_vars {
            let desc = self.global_jvm_descriptor(name, ir_type);
            let outer = if let IrType::Array(_, n) = ir_type { *n } else { 0 };
            let inner = if self.global_uses_object_array.contains(name) {
                if let Some(offsets) = self.global_struct_offset_sets.get(name) {
                    if offsets.is_empty() { 1 } else { offsets.len() }
                } else {
                    if let IrType::Array(inner, _) = ir_type {
                        if let IrType::Array(_, int_slots) = inner.as_ref() {
                            (int_slots * 4) / 8
                        } else { 1 }
                    } else { 1 }
                }
            } else { 0 };

            if outer > 0 && inner > 0 {
                static_fields.push_str(&format!("    static Object[] {} = new Object[{}];\n", name, outer));
                static_init.push_str(&format!(
                    "        for (int __i = 0; __i < {}; __i++) {}[__i] = new Object[{}];\n",
                    outer, name, inner
                ));
            } else if outer > 0 {
                let jtype = Self::desc_to_java_type(&desc);
                static_fields.push_str(&format!("    static {}[] {};\n", jtype, name));
            } else {
                let jtype = Self::desc_to_java_type(&desc);
                static_fields.push_str(&format!("    static {} {};\n", jtype, name));
            }
        }

        format!(r#"import java.io.*;
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

    public static int puts(String s) {{
        System.out.println(s);
        System.out.flush();
        return s.length();
    }}

    public static byte[] malloc(int size) {{
        return new byte[size];
    }}

    public static void free(byte[] ptr) {{}}

    public static int printf(String format, int value) {{
        System.out.print(format.replace("%d", String.valueOf(value))
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

    public static int map_put_jvm(String name, String value) {{
        synchronized (store) {{ store.put(name, value); return 1; }}
    }}

    public static String map_get_jvm(String name) {{
        synchronized (store) {{ return store.get(name); }}
    }}

    public static int map_has_jvm(String name) {{
        synchronized (store) {{ return store.containsKey(name) ? 1 : 0; }}
    }}

    public static int map_remove_jvm(String name) {{
        synchronized (store) {{ return store.remove(name) != null ? 1 : 0; }}
    }}

    public static int map_size_jvm() {{
        synchronized (store) {{ return store.size(); }}
    }}

    public static String map_key_jvm(int i) {{
        synchronized (store) {{
            int idx = 0;
            for (String k : store.keySet()) {{
                if (idx == i) return k;
                idx++;
            }}
            return "";
        }}
    }}

    public static String map_list_jvm() {{
        synchronized (store) {{ return String.join(",", store.keySet()); }}
    }}
}}
"#,
            static_fields = static_fields,
            static_init = static_init,
        )
    }

    fn desc_to_java_type(desc: &str) -> &str {
        match desc {
            "I" => "int",
            "Z" => "int",
            "[B" => "byte[]",
            _ => "int",
        }
    }
}
