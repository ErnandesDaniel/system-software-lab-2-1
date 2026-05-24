<?php

/**
 * PHP FFI SHM client + interactive CLI — всё в одном файле
 *
 * Использование:
 *   1) cargo run -- cli_app/server.mylang -o output -t jvm   # скомпилировать сервер
 *   2) php labs-examples/vitrual-machines/lab-4/cli_app.php   # запустить CLI
 */

// ─── SHMClient ───────────────────────────────────────────────────────────────

class SHMClient {
    private FFI $ffi;
    private FFI\CData $ptr;
    private FFI\CData $hFile;
    private FFI\CData $hMap;
    private FFI\CData $hEvent;
    private int $shmSize = 4096;

    const OP_CREATE = 0;
    const OP_GET    = 1;
    const OP_SET    = 2;
    const OP_DELETE = 3;
    const OP_LIST   = 4;
    const OP_EXIT   = 5;

    public function __construct(string $filePath = 'mylang_shm.dat') {
        $this->ffi = FFI::cdef("
            void* CreateFileA(const char* lpFileName, uint32_t dwDesiredAccess,
                uint32_t dwShareMode, void* lpSecurityAttributes,
                uint32_t dwCreationDisposition, uint32_t dwFlagsAndAttributes,
                void* hTemplateFile);
            void* CreateFileMappingA(void* hFile, void* lpAttributes,
                uint32_t flProtect, uint32_t dwMaximumSizeHigh,
                uint32_t dwMaximumSizeLow, const char* lpName);
            void* MapViewOfFile(void* hFileMappingObject,
                uint32_t dwDesiredAccess, uint32_t dwFileOffsetHigh,
                uint32_t dwFileOffsetLow, uintptr_t dwNumberOfBytesToMap);
            int UnmapViewOfFile(void* lpBaseAddress);
            int CloseHandle(void* hObject);

            void* CreateEventA(void* lpEventAttributes, int bManualReset,
                int bInitialState, const char* lpName);
            int SetEvent(void* hEvent);
            int ResetEvent(void* hEvent);
        ", "kernel32.dll");

        $hFile = $this->ffi->CreateFileA(
            $filePath,
            0x80000000 | 0x40000000,
            1 | 2,
            null,
            4,  // OPEN_ALWAYS
            128,
            null
        );
        if (FFI::isNull($hFile)) throw new RuntimeException("CreateFileA failed");
        $this->hFile = $hFile;

        $hMap = $this->ffi->CreateFileMappingA($hFile, null, 4, 0, $this->shmSize, null);
        if (FFI::isNull($hMap)) throw new RuntimeException("CreateFileMappingA failed");
        $this->hMap = $hMap;

        $ptr = $this->ffi->MapViewOfFile($hMap, 0xF001F, 0, 0, $this->shmSize);
        if (FFI::isNull($ptr)) throw new RuntimeException("MapViewOfFile failed");
        $this->ptr = $ptr;

        $this->hEvent = $this->ffi->CreateEventA(null, 1, 0, "MyLangSHMEvent");
        if (FFI::isNull($this->hEvent)) throw new RuntimeException("CreateEventA failed");
    }

    public function create(string $name, string $value): ?string { return $this->request(self::OP_CREATE, $name, $value); }
    public function get(string $name): ?string { return $this->request(self::OP_GET, $name); }
    public function set(string $name, string $value): ?string { return $this->request(self::OP_SET, $name, $value); }
    public function delete(string $name): ?string { return $this->request(self::OP_DELETE, $name); }
    public function list(): ?string { return $this->request(self::OP_LIST); }
    public function shutdown(): void { $this->request(self::OP_EXIT); }

    private function request(int $opcode, string $key = '', string $value = ''): ?string {
        $mem = FFI::cast('char[4096]', $this->ptr);
        $mem[4] = chr($opcode);
        $pos = 5;
        $klen = strlen($key);
        for ($i = 0; $i < $klen; $i++) $mem[$pos++] = $key[$i];
        $mem[$pos++] = "\0";
        $vlen = strlen($value);
        for ($i = 0; $i < $vlen; $i++) $mem[$pos++] = $value[$i];
        $mem[$pos] = "\0";

        $intPtr = FFI::cast('int*', $this->ptr);
        $intPtr[0] = 1;
        $this->ffi->SetEvent($this->hEvent);
        $this->waitForState(2);

        $resultByte = ord($mem[4]);
        $payload = '';
        for ($i = 5; $i < $this->shmSize; $i++) {
            $c = $mem[$i];
            if ($c === "\0") break;
            $payload .= $c;
        }
        $intPtr[0] = ($opcode === self::OP_EXIT) ? 3 : 0;
        if ($resultByte !== 0) throw new RuntimeException($payload ?: "Unknown error");
        return $payload !== '' ? $payload : null;
    }

    private function waitForState(int $target): void {
        $intPtr = FFI::cast('int*', $this->ptr);
        $timeout = 20000;
        while ($timeout-- > 0) {
            if ($intPtr[0] === $target) return;
            usleep(5000);
        }
        throw new RuntimeException("Timeout waiting for state $target");
    }

    public function close(): void {
        $this->ffi->CloseHandle($this->hEvent);
        $this->ffi->UnmapViewOfFile($this->ptr);
        $this->ffi->CloseHandle($this->hMap);
        $this->ffi->CloseHandle($this->hFile);
    }
}

// ─── CLI ─────────────────────────────────────────────────────────────────────

function printHelp(): void {
    echo "
PHP FFI -> JVM Daemon (Shared Memory)
=====================================

Commands:
  create <key> <value>      Create entry
  get <key>                 Get value
  set <key> <value>         Update value
  delete <key>              Delete key
  list                      List keys
  exit                      Stop daemon and exit
  help                      This help
";
}

function startDaemon(): void {
    echo "[INFO] Starting JVM daemon...\n";
    $cmd = sprintf('powershell -Command "Start-Process -WindowStyle Hidden java \'-cp output;output\\lib\\jna-5.14.0.jar RuntimeStub\'"');
    shell_exec($cmd);
    for ($i = 0; $i < 30; $i++) {
        if (@file_exists('mylang_shm.dat')) {
            echo "[OK] Daemon started\n";
            return;
        }
        usleep(200000);
    }
    echo "[ERR] Failed to start JVM daemon\n";
    echo "  Try: java -cp \"output;output/lib/jna-5.14.0.jar\" RuntimeStub\n";
    exit(1);
}

function connect(): ?SHMClient {
    try { return new SHMClient(); }
    catch (Exception $e) { return null; }
}

function shmIsStale(): bool {
    if (!@file_exists('mylang_shm.dat')) return false;
    $out = shell_exec('tasklist /NH /FI "IMAGENAME eq java.exe" 2>NUL');
    return $out === null || trim($out) === '' || !str_contains($out, 'java');
}

function main(): void {
    echo "=== PHP FFI -> JVM Daemon ===\n\n";
    if (shmIsStale()) { @unlink('mylang_shm.dat'); echo "[INFO] Removed stale SHM file\n"; }
    if (!@file_exists('mylang_shm.dat')) startDaemon();

    $shm = connect();
    if ($shm === null) { echo "[ERR] Failed to connect\n"; exit(1); }
    echo "[OK] Connected\n\n";

    while (true) {
        echo "> ";
        $input = fgets(STDIN);
        if ($input === false) break;
        $line = trim($input);
        if ($line === '') continue;
        $parts = explode(' ', $line);
        $cmd = array_shift($parts);
        try {
            switch ($cmd) {
                case 'exit':
                case 'quit':
                    $shm->shutdown();
                    echo "Bye!\n";
                    break 2;
                case 'start':
                    startDaemon();
                    break;
                case 'help':
                    printHelp();
                    break;
                case 'create':
                    $shm->create(array_shift($parts) ?? '', implode(' ', $parts));
                    echo "  ok\n";
                    break;
                case 'get':
                    $r = $shm->get(array_shift($parts) ?? '');
                    echo "  value: " . ($r ?? '(empty)') . "\n";
                    break;
                case 'set':
                    $shm->set(array_shift($parts) ?? '', implode(' ', $parts));
                    echo "  ok\n";
                    break;
                case 'delete':
                    $shm->delete(array_shift($parts) ?? '');
                    echo "  ok\n";
                    break;
                case 'list':
                    $r = $shm->list();
                    if ($r === null || $r === '') {
                        echo "  (no keys)\n";
                    } else {
                        echo "  keys:\n";
                        foreach (explode(',', $r) as $e) {
                            echo "    - $e\n";
                        }
                    }
                    break;
                default:
                    echo "Unknown: $cmd (type 'help')\n";
            }
        } catch (Exception $e) { echo "  ERROR: " . $e->getMessage() . "\n"; }
    }
    $shm->close();
}

main();
