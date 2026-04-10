bits 64
default rel
section .text

global main
extern add

main:
    push rbp
    mov rbp, rsp
    sub rsp, 48
BB_0:
    mov eax, 1
    mov [rbp + -16], eax
    mov eax, 2
    mov [rbp + -24], eax
    mov ecx, [rbp + -16]
    mov edx, [rbp + -24]
    sub rsp, 32
    call add
    add rsp, 32
    mov [rbp + -32], eax
    mov eax, [rbp + -32]
    mov [rbp + -8], eax
