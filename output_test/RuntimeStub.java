import java.io.*;
import java.util.*;

public class RuntimeStub {
    private static HashMap<String, String> store = new HashMap<>();
    private static Random random = new Random();
    private static ArrayList<byte[]> gc_data = new ArrayList<>();

    // --- Global static fields ---
    static byte[] row_buf;
    static int[] col_pipe;
    static int[] co;
    static byte[] tv_buf;
    static int[] tv_id;
    static int[] tv_name;
    static int tv_n;
    static byte[] pp_buf;
    static int[] pp_id;
    static int[] pp_surname;
    static int[] pp_name;
    static int[] pp_patr;
    static int[] pp_bday;
    static int pp_n;
    static byte[] st_buf;
    static int[] st_pid;
    static int[] st_form;
    static int[] st_fac;
    static int[] st_course;
    static int st_n;
    static byte[] sd_buf;
    static int[] sd_pid;
    static int[] sd_group;
    static int[] sd_start;
    static int sd_n;
    static byte[] vv_buf;
    static int[] vv_tid;
    static int[] vv_pid;
    static int[] vv_mark;
    static int[] vv_date;
    static int vv_n;
    static byte[] gp_buf;
    static int[] gp_pid;
    static int[] gp_group;
    static int[] gp_dept;
    static int gp_n;
    static int[] w;

    static {
        col_pipe = new int[259];
        co = new int[20];
        tv_id = new int[10];
        tv_name = new int[10];
        pp_id = new int[100];
        pp_surname = new int[100];
        pp_name = new int[100];
        pp_patr = new int[100];
        pp_bday = new int[100];
        st_pid = new int[100];
        st_form = new int[100];
        st_fac = new int[100];
        st_course = new int[100];
        sd_pid = new int[100];
        sd_group = new int[100];
        sd_start = new int[100];
        vv_tid = new int[100];
        vv_pid = new int[100];
        vv_mark = new int[100];
        vv_date = new int[100];
        gp_pid = new int[50];
        gp_group = new int[50];
        gp_dept = new int[50];
        w = new int[100];
    }

    public static void main(String[] args) {
        int result = Main.call();
        System.exit(result);
    }

    // --- Standard IO ---

    public static int getchar() throws IOException {
        return System.in.read();
    }

    public static int putchar(int c) {
        System.out.print((char) c);
        System.out.flush();
        return c;
    }

    public static int puts(byte[] s) {
        System.out.println(new String(s));
        System.out.flush();
        return s.length;
    }

    public static byte[] malloc(int size) {
        byte[] mem = new byte[size];
        gc_data.add(mem);
        return mem;
    }

    public static void free(byte[] ptr) {
        if (ptr == null) return;
        gc_data.remove(ptr);
    }

    public static int printf(byte[] format, int value) {
        String fmt = new String(format);
        StringBuilder sb = new StringBuilder();
        for (int __i = 0; __i < fmt.length(); __i++) {
            char c = fmt.charAt(__i);
            if (c != '%') { sb.append(c); continue; }
            __i++;
            if (__i >= fmt.length()) { sb.append(c); break; }
            if (fmt.charAt(__i) == 'd') { sb.append(value); continue; }
            if (fmt.charAt(__i) == 'c') { sb.append((char) value); continue; }
            if (fmt.charAt(__i) == 's') { sb.append(value); continue; }
            boolean zeroPad = false;
            if (fmt.charAt(__i) == '0') { zeroPad = true; __i++; }
            int width = 0;
            while (__i < fmt.length() && fmt.charAt(__i) >= '0' && fmt.charAt(__i) <= '9') {
                width = width * 10 + (fmt.charAt(__i) - '0');
                __i++;
            }
            if (__i < fmt.length() && (fmt.charAt(__i) == 'd' || fmt.charAt(__i) == 'c' || fmt.charAt(__i) == 's')) {
                String num = String.valueOf(fmt.charAt(__i) == 'c' ? (char) value : value);
                while (num.length() < width) num = zeroPad ? "0" + num : " " + num;
                sb.append(num);
            } else {
                __i--;
                sb.append('%');
            }
        }
        System.out.print(sb.toString().replace("\\n", "\n").replace("\\t", "\t"));
        System.out.flush();
        return value;
    }

    public static int rand() {
        return random.nextInt();
    }

    public static void srand(int seed) {
        random = new Random(seed);
    }

    public static int time(int dummy) {
        return (int)(System.currentTimeMillis() / 1000);
    }

    public static void Sleep(int ms) {
        try { Thread.sleep(ms); } catch (InterruptedException e) {}
    }

    // --- Map functions (JVM) ---

    public static int map_put_jvm(byte[] name, byte[] value) {
        synchronized (store) { store.put(new String(name), new String(value)); return 1; }
    }

    public static byte[] map_get_jvm(byte[] name) {
        synchronized (store) { String v = store.get(new String(name)); return v != null ? v.getBytes() : new byte[0]; }
    }

    public static int map_has_jvm(byte[] name) {
        synchronized (store) { return store.containsKey(new String(name)) ? 1 : 0; }
    }

    public static int map_remove_jvm(byte[] name) {
        synchronized (store) { return store.remove(new String(name)) != null ? 1 : 0; }
    }

    public static int map_size_jvm() {
        synchronized (store) { return store.size(); }
    }

    public static byte[] map_key_jvm(int i) {
        synchronized (store) {
            int idx = 0;
            for (String k : store.keySet()) {
                if (idx == i) return k.getBytes();
                idx++;
            }
            return new byte[0];
        }
    }

    public static byte[] map_list_jvm() {
        synchronized (store) { return String.join(",", store.keySet()).getBytes(); }
    }
}
