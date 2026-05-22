import java.io.*;
import java.nio.*;
import java.nio.channels.*;
import java.nio.charset.*;
import java.util.*;
import com.sun.jna.*;
import com.sun.jna.win32.*;

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
    private static final byte OP_EXIT   = 5;

    private static final byte RES_OK    = 0;
    private static final byte RES_ERR   = 1;

    private static final int INFINITE = -1;
    private static final int WAIT_TIMEOUT = 0x00000102;

    private final MappedByteBuffer buf;
    private final Map<String, String> store = new HashMap<>();
    private final Pointer hEvent;

    public MainServer() throws Exception {
        System.out.println("[JVM Daemon] Initializing...");
        RandomAccessFile file = new RandomAccessFile("mylang_shm.dat", "rw");
        file.setLength(SHM_SIZE);
        FileChannel ch = file.getChannel();
        buf = ch.map(FileChannel.MapMode.READ_WRITE, 0, SHM_SIZE);
        buf.order(ByteOrder.LITTLE_ENDIAN);
        ch.close();

        hEvent = Kernel32.INSTANCE.CreateEventA(null, true, false, "MyLangSHMEvent");
        if (hEvent == null) {
            throw new RuntimeException("CreateEventA failed");
        }

        writeState(STATE_IDLE);
        System.out.println("[JVM Daemon] Ready. PID: " + ProcessHandle.current().pid());
    }

    public void run() throws Exception {
        while (true) {
            Kernel32.INSTANCE.WaitForSingleObject(hEvent, 2000);
            Kernel32.INSTANCE.ResetEvent(hEvent);

            int state = readState();
            if (state == STATE_REQ) {
                handleRequest();
                writeState(STATE_DONE);
            } else if (state == STATE_EXIT) {
                break;
            }
        }
        shutdown();
    }

    private int readState() {
        return buf.getInt(0);
    }

    private void writeState(int state) {
        buf.putInt(0, state);
    }

    private int readByte(int pos) {
        return buf.get(pos) & 0xFF;
    }

    private int indexOfNull(int start) {
        for (int i = start; i < SHM_SIZE; i++) {
            if (buf.get(i) == 0) return i;
        }
        return SHM_SIZE;
    }

    private String readStr(int pos) {
        ByteBuffer slice = buf.duplicate();
        slice.position(pos);
        int len = 0;
        while (pos + len < SHM_SIZE && slice.get() != 0) len++;
        byte[] bytes = new byte[len];
        buf.position(pos);
        buf.get(bytes);
        return new String(bytes, StandardCharsets.UTF_8);
    }

    private void writeResp(byte result, String payload) {
        try {
            byte[] pbytes = payload.getBytes(StandardCharsets.UTF_8);
            int maxLen = SHM_SIZE - 6;
            if (pbytes.length > maxLen) pbytes = java.util.Arrays.copyOf(pbytes, maxLen);

            buf.position(4);
            buf.put(result);
            buf.put(pbytes);
            buf.put((byte) 0);
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

    private void shutdown() {
        Kernel32.INSTANCE.CloseHandle(hEvent);
        System.out.println("[JVM Daemon] Shutdown complete.");
    }

    public static void main(String[] args) throws Exception {
        new MainServer().run();
    }

    private interface Kernel32 extends StdCallLibrary {
        Kernel32 INSTANCE = Native.load("kernel32", Kernel32.class, W32APIOptions.DEFAULT_OPTIONS);

        Pointer CreateEventA(Pointer lpEventAttributes, boolean bManualReset, boolean bInitialState, String lpName);
        int WaitForSingleObject(Pointer hHandle, int dwMilliseconds);
        boolean SetEvent(Pointer hEvent);
        boolean ResetEvent(Pointer hEvent);
        boolean CloseHandle(Pointer hObject);
    }
}
