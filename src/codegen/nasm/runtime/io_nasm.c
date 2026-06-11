#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

void *xmalloc(size_t size) {
    void *p = malloc(size);
    if (!p && size > 0) {
        puts("xmalloc failed");
        exit(1);
    }
    return p;
}

void xfree(void *p) {
    free(p);
}

// Wrapper around standard C fread
size_t fread_nasm(void *buf, size_t size, size_t count, FILE *stream) {
    return fread(buf, size, count, stream);
}

// Wrapper around standard C fseek
int fseek_nasm(FILE *stream, long offset, int whence) {
    return fseek(stream, offset, whence);
}

// Read a 32-bit little-endian integer from buf + offset
int32_t read_le32_nasm(const void *buf, int offset) {
    const uint8_t *p = (const uint8_t *)buf + offset;
    return (int32_t)(p[0] | (p[1] << 8) | (p[2] << 16) | (p[3] << 24));
}

// Read a 16-bit little-endian integer from buf + offset
int32_t read_le16_nasm(const void *buf, int offset) {
    const uint8_t *p = (const uint8_t *)buf + offset;
    return (int32_t)(p[0] | (p[1] << 8));
}

// Read a single byte from buf + offset
int32_t read_i8_nasm(const void *buf, int offset) {
    return (int32_t)((const uint8_t *)buf)[offset];
}

// Wrapper around standard C fwrite
size_t fwrite_nasm(const void *buf, size_t size, size_t count, FILE *stream) {
    return fwrite(buf, size, count, stream);
}

// Coroutine runtime stubs for platforms without ucontext.h (e.g. Windows).
// On Linux these are provided by coro_linux.c.
#ifdef _WIN32

#include <windows.h>

#define MAX_COROS 16
#define STACK_SIZE (64 * 1024)

// Simplified coroutine context (stack + instruction pointer)
typedef struct {
    void (*fn)(void);
    void *stack;
    int  active;
    int  finished;
    int  saved_rsp;  // Rough approximation — just tracks completion
} CoroStub;

static CoroStub coros[MAX_COROS];
static int coro_n = 0;
static int current = 0;
static int (*scheduler_fn)(void) = NULL;

int create_coroutine_nasm(void (*fn)(void)) {
    if (coro_n < MAX_COROS) {
        coros[coro_n].fn = fn;
        coros[coro_n].stack = malloc(STACK_SIZE);
        coros[coro_n].active = 1;
        coros[coro_n].finished = 0;
        coro_n++;
    }
    return coro_n - 1;
}

void init_coroutine_runtime_nasm(void) {
    coro_n = 0;
    current = 0;
    scheduler_fn = NULL;
}

void set_coroutine_scheduler_nasm(int (*fn)(void)) {
    scheduler_fn = fn;
}

int get_current_coroutine_id_nasm(void) {
    return current;
}

// Forward declare the setjmp/longjmp trampoline
static int coro_trampoline_active = 0;

void run_coroutine_runtime_nasm(void) {
    // Simple cooperative round-robin: each coroutine runs until it returns
    // (its function terminates). We iterate until all are finished.
    int all_done = 0;
    while (!all_done) {
        all_done = 1;
        for (int i = 0; i < coro_n; i++) {
            if (coros[i].finished) continue;
            all_done = 0;
            current = i;
            coros[i].fn();
            coros[i].finished = 1;
        }
    }
}

#endif
