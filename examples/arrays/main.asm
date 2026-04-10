bits 64
default rel
section .text

global main
main:
    push rbp
    mov rbp, rsp
    sub rsp, 48
BB_0:
    mov eax, 0
    mov [rbp + -16], eax
    mov rax, 0
    mov eax, [rbp + -16]
    mov ebx, eax
    mov eax, [rax + rbx * 4]
    mov [rbp + -8], eax
    mov eax, 1
    mov [rbp + -32], eax
    mov eax, [rbp + -32]
    mov [rbp + -8], eax
