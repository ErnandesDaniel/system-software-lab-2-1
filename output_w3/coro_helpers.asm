bits 64
default rel

section .data
co_states times 8 dq 0
state_worker times 9 dd 0

section .text
global resume_coroutine_nasm
resume_coroutine_nasm:
    ; rcx = index
    lea rax, [rel co_states]
    mov rax, [rax + rcx * 8]
    test rax, rax
    jz .empty
    mov rcx, rax
    mov eax, [rcx]
    cmp eax, -1
    jne .go
    mov eax, 1
    ret
.go:
    push rbp
    mov rbp, rsp
    sub rsp, 32
    call [rcx + 8]
    mov eax, [rcx + 16]
    leave
    ret
.empty:
    mov eax, 1
    ret

global create_coroutine_nasm
create_coroutine_nasm:
    mov dword [rcx], 0
    mov [rcx + 8], rdx
    mov dword [rcx + 16], 0
    ret

global coro_init_nasm
extern worker
coro_init_nasm:
    push rbp
    mov rbp, rsp
    lea rcx, [rel state_worker]
    lea rdx, [rel worker]
    sub rsp, 32
    call create_coroutine_nasm
    add rsp, 32
    lea rax, [rel co_states]
    lea rcx, [rel state_worker]
    mov [rax + 0], rcx
    leave
    ret

