bits 64
default rel
section .text

global main

extern coro_init
extern puts
extern run

main:
    push rbp
    mov rbp, rsp
    sub rsp, 48
main_BB0:
    lea rax, [main_str_0]
    mov [rbp + -8], rax
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -16], eax
    sub rsp, 32
    call coro_init
    add rsp, 32
    sub rsp, 32
    call run
    add rsp, 32
    mov [rbp + -24], eax
    mov eax, 0
    mov [rbp + -32], eax
    mov eax, [rbp + -32]
    leave
    ret

section .data
main_str_0 db 83, 116, 97, 114, 116, 0
