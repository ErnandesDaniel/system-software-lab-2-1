extern buf
bits 64
default rel
section .text

bits 64
default rel
section .text

global main
extern puts

main:
    push rbp
    mov rbp, rsp
    sub rsp, 32
BB_0:
    lea rax, [main_str_0]
    mov [rbp + -8], rax
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -16], eax
    mov eax, 0
    mov [rbp + -24], eax
    mov eax, [rbp + -24]
    leave
    ret

section .data
main_str_0 db 111, 107, 0
