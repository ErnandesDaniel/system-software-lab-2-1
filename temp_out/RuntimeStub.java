import java.io.*;
import java.util.*;

public class RuntimeStub {
    private static HashMap<String, String> store = new HashMap<>();
    private static Random random = new Random();

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

    public static int puts(String s) {
        System.out.println(s);
        System.out.flush();
        return s.length();
    }

    public static int printf(String format, int value) {
        System.out.print(format.replace("%d", String.valueOf(value))
                                .replace("%c", String.valueOf((char) value))
                                .replace("%s", String.valueOf(value))
                                .replace("\\n", "\n")
                                .replace("\\t", "\t"));
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

    public static int map_put_jvm(String name, String value) {
        synchronized (store) { store.put(name, value); return 1; }
    }

    public static String map_get_jvm(String name) {
        synchronized (store) { return store.get(name); }
    }

    public static int map_has_jvm(String name) {
        synchronized (store) { return store.containsKey(name) ? 1 : 0; }
    }

    public static int map_remove_jvm(String name) {
        synchronized (store) { return store.remove(name) != null ? 1 : 0; }
    }

    public static int map_size_jvm() {
        synchronized (store) { return store.size(); }
    }

    public static String map_key_jvm(int i) {
        synchronized (store) {
            int idx = 0;
            for (String k : store.keySet()) {
                if (idx == i) return k;
                idx++;
            }
            return "";
        }
    }

    public static String map_list_jvm() {
        synchronized (store) { return String.join(",", store.keySet()); }
    }
}
