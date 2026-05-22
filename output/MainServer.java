import java.io.*;
import java.nio.*;
import java.nio.charset.*;
import java.util.*;

public class MainServer {
    private static final int SHM_SIZE = 4096;
    private static final int STATE_IDLE = 0;
    private static final int STATE_REQ  = 1;
    private static final int STATE_DONE = 2;
    private static final int STATE_EXIT = 3;

    private static final byte OP_CREATE = 0;
    private static final byte OP_GET    = 1;
    private static final byte OP_SET    = 2;
    private static final byte OP_DELETE = 3;
    private static final byte OP_LIST   = 4;
    private static final byte OP_EXEC   = 5;
    private static final byte OP_EXIT   = 6;

    private static final byte RES_OK    = 0;
    private static final byte RES_ERR   = 1;

    private final RandomAccessFile file;
    private final Map<String, String> store = new HashMap<>();

    public MainServer() throws Exception {
        System.out.println("[JVM Daemon] Initializing...");
        file = new RandomAccessFile("mylang_shm.dat", "rw");
        file.setLength(SHM_SIZE);
        writeState(STATE_IDLE);
        System.out.println("[JVM Daemon] Ready. PID: " + ProcessHandle.current().pid());
    }

    public void run() throws Exception {
        while (true) {
            int state = readState();
            if (state == STATE_REQ) {
                handleRequest();
                writeState(STATE_DONE);
            } else if (state == STATE_EXIT) {
                break;
            }
            Thread.sleep(10);
        }
        shutdown();
    }

    private final byte[] stBuf = new byte[4];

    private int readState() throws IOException {
        synchronized (file) {
            file.seek(0);
            file.readFully(stBuf);
        }
        return ByteBuffer.wrap(stBuf).order(ByteOrder.LITTLE_ENDIAN).getInt();
    }

    private void writeState(int state) throws IOException {
        byte[] b = ByteBuffer.allocate(4).order(ByteOrder.LITTLE_ENDIAN).putInt(state).array();
        synchronized (file) {
            file.seek(0);
            file.write(b);
        }
    }

    private int readByte(int pos) throws IOException {
        synchronized (file) {
            file.seek(pos);
            return file.read();
        }
    }

    private String readStr(int pos) throws IOException {
        byte[] buf = new byte[SHM_SIZE - pos];
        synchronized (file) {
            file.seek(pos);
            file.read(buf);
        }
        int len = 0;
        while (len < buf.length && buf[len] != 0) len++;
        return new String(buf, 0, len, StandardCharsets.UTF_8);
    }

    private int indexOfNull(int start) throws IOException {
        byte[] buf = new byte[SHM_SIZE - start];
        synchronized (file) {
            file.seek(start);
            file.read(buf);
        }
        for (int i = 0; i < buf.length; i++) {
            if (buf[i] == 0) return start + i;
        }
        return start + buf.length;
    }

    private void writeResp(byte result, String payload) {
        try {
            byte[] pbytes = payload.getBytes(StandardCharsets.UTF_8);
            int maxLen = SHM_SIZE - 6;
            if (pbytes.length > maxLen) pbytes = java.util.Arrays.copyOf(pbytes, maxLen);

            synchronized (file) {
                file.seek(4);
                file.write(result);
                file.write(pbytes);
                file.write(0);
            }
        } catch (Exception e) {
            System.err.println("[JVM] Write error: " + e.getMessage());
        }
    }

    private void handleRequest() {
        try {
            int opcode = readByte(4);
            int keyEnd = indexOfNull(5);
            String key = readStr(5);
            String value = "";
            if (keyEnd + 1 < SHM_SIZE) {
                value = readStr(keyEnd + 1);
            }

            switch (opcode) {
                case OP_CREATE -> handleCreate(key, value);
                case OP_GET    -> handleGet(key);
                case OP_SET    -> handleSet(key, value);
                case OP_DELETE -> handleDelete(key);
                case OP_LIST   -> handleList();
                case OP_EXEC   -> handleExec(key, value);
                case OP_EXIT   -> writeResp(RES_OK, "");
                default -> writeResp(RES_ERR, "Unknown opcode: " + opcode);
            }
        } catch (Exception e) {
            writeResp(RES_ERR, e.getMessage() != null ? e.getMessage() : "Error");
        }
    }

    private void handleCreate(String name, String value) {
        if (name.isEmpty()) { writeResp(RES_ERR, "Key name required"); return; }
        synchronized (store) {
            if (store.containsKey(name)) {
                writeResp(RES_ERR, "Key '" + name + "' already exists");
                return;
            }
            store.put(name, value);
        }
        writeResp(RES_OK, "");
    }

    private void handleGet(String name) {
        if (name.isEmpty()) { writeResp(RES_ERR, "Key name required"); return; }
        String value;
        synchronized (store) {
            value = store.get(name);
        }
        if (value == null) {
            writeResp(RES_ERR, "Key '" + name + "' not found");
        } else {
            writeResp(RES_OK, value);
        }
    }

    private void handleSet(String name, String value) {
        if (name.isEmpty()) { writeResp(RES_ERR, "Key name required"); return; }
        synchronized (store) {
            if (!store.containsKey(name)) {
                writeResp(RES_ERR, "Key '" + name + "' not found");
                return;
            }
            store.put(name, value);
        }
        writeResp(RES_OK, "");
    }

    private void handleDelete(String name) {
        synchronized (store) {
            if (store.remove(name) == null) {
                writeResp(RES_ERR, "Key '" + name + "' not found");
                return;
            }
        }
        writeResp(RES_OK, "");
    }

    private void handleList() {
        String[] names;
        synchronized (store) {
            names = store.keySet().toArray(new String[0]);
        }
        writeResp(RES_OK, String.join(",", names));
    }

    private void handleExec(String func, String argsStr) {
        if (func.isEmpty()) { writeResp(RES_ERR, "No function name"); return; }
        try {
            String[] parts = argsStr.split(",");
            int[] args = new int[parts.length];
            for (int i = 0; i < parts.length; i++)
                args[i] = Integer.parseInt(parts[i].trim());

            int result;
            switch (func) {
                case "square" -> { if (args.length != 1) throw new IllegalArgumentException("square needs 1 arg"); result = args[0] * args[0]; }
                case "add" -> { if (args.length < 2) throw new IllegalArgumentException("add needs 2+ args"); result = java.util.Arrays.stream(args).sum(); }
                default -> { writeResp(RES_ERR, "Unknown function: " + func); return; }
            }
            writeResp(RES_OK, String.valueOf(result));
        } catch (Exception e) {
            writeResp(RES_ERR, e.getMessage() != null ? e.getMessage() : "Error");
        }
    }

    private void shutdown() {
        try { file.close(); System.out.println("[JVM Daemon] Shutdown complete."); }
        catch (Exception e) { System.err.println("[JVM] Shutdown error: " + e.getMessage()); }
    }

    public static void main(String[] args) throws Exception {
        new MainServer().run();
    }
}
