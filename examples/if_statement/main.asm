bits 64
default rel
section .text

global main
main:
    push rbp
    mov rbp, rsp
    sub rsp, 64
BB_1:
    mov eax, 1
    mov [rbp + -24], eax
    mov eax, [rbp + -24]
    mov [rbp + -16], eax
BB_2:
BB_2:
    mov eax, 5
    mov [rbp + -32], eax
    mov eax, [rbp + -32]
    mov [rbp + -8], eax
    mov eax, 0
    mov [rbp + -40], eax
    mov eax, [rbp + -8]
    mov eax, [rbp + -40]
    mov ebx, eax
    cmp eax, ebx
    setg al
    movzx eax, al
    mov [rbp + -48], eax
    mov eax, [rbp + -48]
    test eax, eax
    jne BB_1
    jmp BB_2
