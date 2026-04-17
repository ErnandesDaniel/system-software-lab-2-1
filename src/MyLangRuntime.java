public class MyLangRuntime {
    public static void println(int x) {
        System.out.println(x);
    }
    
    public static void putchar(int c) {
        System.out.print((char) c);
    }
    
    public static int getchar() throws java.io.IOException {
        return System.in.read();
    }
    
    public static int rand() {
        return (int) (Math.random() * Integer.MAX_VALUE);
    }
    
    public static void srand(int seed) {
        // Not directly supported in Java, but we can use java.util.Random
    }
    
    public static long time() {
        return System.currentTimeMillis() / 1000;
    }
}