#include <signal.h>
#include <stdio.h>
#include <sys/time.h>
#include <time.h>
#include <ucontext.h>
#include <stdlib.h>
#include <unistd.h>

#define MAX 32
#define STACK_SIZE (1024 * 256)
#define QUANTUM_US 20000

typedef struct {
    ucontext_t ctx;
    int active;
    int finished;
    void (*fn)(void);
} Coro;

static Coro coros[MAX];
static int n = 0;
static int cur = 0;
static ucontext_t tramp_ctx;
static ucontext_t main_ctx;
static int (*scheduler_fn)(void) = NULL;
static struct itimerval quant = { {0, QUANTUM_US}, {0, QUANTUM_US} };

int get_current_coroutine_id_nasm(void) {
    return cur;
}

static void coro_entry(int id) {
    coros[id].fn();
    coros[id].finished = 1;
    setcontext(&tramp_ctx);
}

static void trampoline(void) {
    while (1) {
        int next = scheduler_fn ? scheduler_fn() : -1;
        if (next < 0 || next >= n || coros[next].finished) break;
        cur = next;
        setitimer(ITIMER_REAL, &quant, NULL);
        swapcontext(&tramp_ctx, &coros[next].ctx);
    }
    setcontext(&main_ctx);
}

static void tick(int sig) {
    if (n == 0 || !scheduler_fn) return;
    if (coros[cur].finished) return;
    swapcontext(&coros[cur].ctx, &tramp_ctx);
}

void Sleep(int ms) {
    struct timespec ts = { .tv_sec = ms / 1000, .tv_nsec = (ms % 1000) * 1000000 };
    nanosleep(&ts, NULL);
}

void set_coroutine_scheduler_nasm(int (*fn)(void)) {
    scheduler_fn = fn;
}

void init_coroutine_runtime_nasm(void) {
    n = 0;
    cur = 0;
    scheduler_fn = NULL;
    struct sigaction sa;
    sa.sa_handler = tick;
    sa.sa_flags = SA_NODEFER;
    sigemptyset(&sa.sa_mask);
    sigaction(SIGALRM, &sa, NULL);
}

int create_coroutine_nasm(void (*fn)(void)) {
    if (n >= MAX) return -1;
    int id = n++;
    coros[id].active = 1;
    coros[id].finished = 0;
    coros[id].fn = fn;
    sigset_t old, m;
    sigemptyset(&m); sigaddset(&m, SIGALRM);
    sigprocmask(SIG_UNBLOCK, &m, &old);
    getcontext(&coros[id].ctx);
    coros[id].ctx.uc_stack.ss_sp = malloc(STACK_SIZE);
    coros[id].ctx.uc_stack.ss_size = STACK_SIZE;
    coros[id].ctx.uc_link = NULL;
    makecontext(&coros[id].ctx, (void (*)())coro_entry, 1, id);
    sigprocmask(SIG_SETMASK, &old, NULL);
    return id;
}

void run_coroutine_runtime_nasm(void) {
    if (n == 0) return;
    cur = 0;
    getcontext(&tramp_ctx);
    tramp_ctx.uc_stack.ss_sp = malloc(STACK_SIZE);
    tramp_ctx.uc_stack.ss_size = STACK_SIZE;
    tramp_ctx.uc_link = NULL;
    makecontext(&tramp_ctx, trampoline, 0);
    sigset_t m;
    sigemptyset(&m); sigaddset(&m, SIGALRM);
    sigprocmask(SIG_UNBLOCK, &m, NULL);
    setitimer(ITIMER_REAL, &quant, NULL);
    swapcontext(&main_ctx, &coros[0].ctx);
}
