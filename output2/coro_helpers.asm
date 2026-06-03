bits 64
default rel

section .data
co_states times 8 dq 0
state_print_once times 14 dd 0
state_print_two times 14 dd 0

section .text
global resume_coroutine
resume_coroutine:
    lea rax, [rel co_states]
    mov rax, [rax + rcx * 8]
    test rax, rax
    jz .empty
    mov rbx, rax
    mov eax, [rbx]
    cmp eax, -1
    jne .go
    mov eax, 1
    ret
.go:
    push rbp
    mov rbp, rsp
    sub rsp, 40
    mov [rbp + 32], rbx
    mov rcx, rbx
    mov rdx, [rbx + 32]
    mov r8,  [rbx + 40]
    mov r9,  [rbx + 48]
    call [rbx + 8]
    mov rbx, [rbp + 32]
    mov eax, [rbx + 16]
    leave
    ret
.empty:
    mov eax, 1
    ret

global create_coroutine
create_coroutine:
    mov dword [rcx], 0
    mov [rcx + 8], rdx
    mov dword [rcx + 16], 0
    mov [rcx + 24], r8
    mov [rcx + 32], r9
    ret

global get_coroutine_state
get_coroutine_state:
    lea rax, [rel co_states]
    mov rax, [rax + rcx * 8]
    test rax, rax
    jz .empty
    mov eax, [rax]
    ret
.empty:
    mov eax, -1
    ret

global set_coroutine_param
set_coroutine_param:
    lea rax, [rel co_states]
    mov rax, [rax + rcx * 8]
    test rax, rax
    jz .empty
    mov [rax + 24], edx
    mov [rax + 32], r8d
.empty:
    ret

global coro_init
extern print_once
extern print_two
coro_init:
    push rbp
    mov rbp, rsp
    lea rcx, [rel state_print_once]
    lea rdx, [rel print_once]
    sub rsp, 32
    call create_coroutine
    add rsp, 32
    lea rax, [rel co_states]
    lea rcx, [rel state_print_once]
    mov [rax + 0], rcx
    lea rcx, [rel state_print_two]
    lea rdx, [rel print_two]
    sub rsp, 32
    call create_coroutine
    add rsp, 32
    lea rax, [rel co_states]
    lea rcx, [rel state_print_two]
    mov [rax + 8], rcx
    leave
    ret

