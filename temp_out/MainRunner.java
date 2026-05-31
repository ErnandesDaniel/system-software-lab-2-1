import java.lang.reflect.Method;
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
