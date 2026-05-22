<?php

class SHMClient {
    private FFI $ffi;
    private FFI\CData $ptr;
    private FFI\CData $hFile;
    private FFI\CData $hMap;
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
        ", "kernel32.dll");

        $hFile = $this->ffi->CreateFileA(
            $filePath,
            0x80000000 | 0x40000000,
            1 | 2,
            null,
            4,
            128,
            null
        );

        if (FFI::isNull($hFile)) {
            throw new RuntimeException("CreateFileA failed");
        }
        $this->hFile = $hFile;

        $hMap = $this->ffi->CreateFileMappingA(
            $hFile,
            null,
            4,
            0,
            $this->shmSize,
            null
        );

        if (FFI::isNull($hMap)) {
            throw new RuntimeException("CreateFileMappingA failed");
        }
        $this->hMap = $hMap;

        $ptr = $this->ffi->MapViewOfFile($hMap, 0xF001F, 0, 0, $this->shmSize);
        if (FFI::isNull($ptr)) {
            throw new RuntimeException("MapViewOfFile failed");
        }
        $this->ptr = $ptr;
    }

    public function create(string $name, string $value): ?string { return $this->request(self::OP_CREATE, $name, $value); }
    public function get(string $name): ?string { return $this->request(self::OP_GET, $name); }
    public function set(string $name, string $value): ?string { return $this->request(self::OP_SET, $name, $value); }
    public function delete(string $name): ?string { return $this->request(self::OP_DELETE, $name); }
    public function list(): ?string { return $this->request(self::OP_LIST); }
    public function shutdown(): void { $this->request(self::OP_EXIT); }

    private function request(int $opcode, string $key = '', string $value = ''): ?string {
        $this->waitForState(0);

        $mem = FFI::cast('char[4096]', $this->ptr);

        // write opcode at offset 4
        $mem[4] = chr($opcode);

        // write key at offset 5
        $pos = 5;
        $klen = strlen($key);
        for ($i = 0; $i < $klen; $i++) {
            $mem[$pos++] = $key[$i];
        }
        $mem[$pos++] = "\0";

        // write value
        $vlen = strlen($value);
        for ($i = 0; $i < $vlen; $i++) {
            $mem[$pos++] = $value[$i];
        }
        $mem[$pos] = "\0";

        // set state to REQUEST
        $intPtr = FFI::cast('int*', $this->ptr);
        $intPtr[0] = 1;

        // wait for DONE
        $this->waitForState(2);

        // read result
        $resultByte = ord($mem[4]);

        // read payload
        $payload = '';
        for ($i = 5; $i < $this->shmSize; $i++) {
            $c = $mem[$i];
            if ($c === "\0") break;
            $payload .= $c;
        }

        // reset state
        $intPtr[0] = ($opcode === self::OP_EXIT) ? 3 : 0;

        if ($resultByte !== 0) {
            throw new RuntimeException($payload ?: "Unknown error");
        }

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
        $this->ffi->UnmapViewOfFile($this->ptr);
        $this->ffi->CloseHandle($this->hMap);
        $this->ffi->CloseHandle($this->hFile);
    }
}
