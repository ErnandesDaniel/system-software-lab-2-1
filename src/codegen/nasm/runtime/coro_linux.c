#include <signal.h>
#include <sys/time.h>
#include <time.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdint.h>

#define MAX 32
#define STACK_SIZE (1024 * 64)
#define QUANTUM_US 20000

typedef struct {
    void *rsp;
} CoroContext;

typedef struct {
    CoroContext ctx;
    int finished;
    void (*fn)(void);
    void *stack;
} Coro;

static Coro coros[MAX];
static int n = 0;
static int cur = 0;
static CoroContext tramp_ctx;
static CoroContext main_ctx;
static int (*volatile scheduler_fn)(void) = NULL;
static struct itimerval quant = { {0, QUANTUM_US}, {0, QUANTUM_US} };
static char tramp_stack[STACK_SIZE];
static int tramp_stack_used = 0;

extern void switch_context_nasm(CoroContext *old_ctx, CoroContext *new_ctx);

int get_current_coroutine_id_nasm(void) {
    return cur;
}

static void coro_entry(void) {
    coros[cur].fn();
    coros[cur].finished = 1;
    switch_context_nasm(&coros[cur].ctx, &tramp_ctx);
}

static void trampoline(void) {
    while (1) {
        int next = -1;
        if (scheduler_fn) {
            next = scheduler_fn();
            if (next < 0 || next >= n || coros[next].finished) {
                next = -1;
            }
        }
        if (next < 0) {
            int start = (cur + 1) % n;
            for (int i = 0; i < n; i++) {
                int c = (start + i) % n;
                if (!coros[c].finished) { next = c; break; }
            }
        }
        if (next < 0) {
            switch_context_nasm(&tramp_ctx, &main_ctx);
            return;
        }
        cur = next;
        setitimer(ITIMER_REAL, &quant, NULL);
        switch_context_nasm(&tramp_ctx, &coros[next].ctx);
    }
}

static void tick(int sig) {
    (void)sig;
    if (n == 0) return;
    if (coros[cur].finished) return;
    struct itimerval zero = {{0, 0}, {0, 0}};
    setitimer(ITIMER_REAL, &zero, NULL);
    switch_context_nasm(&coros[cur].ctx, &tramp_ctx);
}

void Sleep(int ms) {
    struct timespec ts = {
        .tv_sec = ms / 1000,
        .tv_nsec = (long)(ms % 1000) * 1000000
    };
    nanosleep(&ts, NULL);
}

void set_coroutine_scheduler_nasm(int (*fn)(void)) {
    scheduler_fn = fn;
}

void init_coroutine_runtime_nasm(void) {
    struct itimerval zero = {{0, 0}, {0, 0}};
    setitimer(ITIMER_REAL, &zero, NULL);
    n = 0;
    cur = 0;
    scheduler_fn = NULL;
    tramp_stack_used = 0;

    struct sigaction sa;
    sa.sa_handler = tick;
    sa.sa_flags = SA_NODEFER;
    sigemptyset(&sa.sa_mask);
    sigaction(SIGALRM, &sa, NULL);
}

int create_coroutine_nasm(void (*fn)(void)) {
    if (n >= MAX) return -1;
    int id = n++;
    coros[id].finished = 0;
    coros[id].fn = fn;

    void *stack = malloc(STACK_SIZE);
    if (!stack) return -1;
    coros[id].stack = stack;

    uint64_t *rsp = (uint64_t *)((char *)stack + STACK_SIZE);

    *(--rsp) = (uint64_t)coro_entry;
    *(--rsp) = 0;
    *(--rsp) = 0;
    *(--rsp) = 0;
    *(--rsp) = 0;
    *(--rsp) = 0;
    *(--rsp) = 0;

    coros[id].ctx.rsp = rsp;
    return id;
}

void run_coroutine_runtime_nasm(void) {
    if (n == 0) return;
    cur = 0;

    uint64_t *rsp = (uint64_t *)(tramp_stack + STACK_SIZE);

    *(--rsp) = (uint64_t)trampoline;
    *(--rsp) = 0;
    *(--rsp) = 0;
    *(--rsp) = 0;
    *(--rsp) = 0;
    *(--rsp) = 0;
    *(--rsp) = 0;
    tramp_ctx.rsp = rsp;

    setitimer(ITIMER_REAL, &quant, NULL);
    switch_context_nasm(&main_ctx, &coros[0].ctx);

    for (int i = 0; i < n; i++) {
        free(coros[i].stack);
        coros[i].stack = NULL;
    }
}
