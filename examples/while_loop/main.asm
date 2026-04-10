bits 64
default rel
section .text

global main
main:
    push rbp
    mov rbp, rsp
    sub rsp, 64
BB_0:
    mov eax, 1
    mov [rbp + -16], eax
    mov eax, [rbp + -16]
    mov [rbp + -8], eax
    jmp BB_1
BB_1:
    mov eax, 5
    mov [rbp + -24], eax
    mov eax, [rbp + -8]
    mov eax, [rbp + -24]
    mov ebx, eax
    cmp eax, ebx
    setl al
    movzx eax, al
    mov [rbp + -32], eax
    mov eax, [rbp + -32]
    test eax, eax
    jne BB_2
    jmp BB_3
BB_2:
    mov eax, 1
    mov [rbp + -40], eax
    mov eax, [rbp + -8]
    mov eax, [rbp + -40]
    mov ebx, eax
    add eax, ebx
    mov [rbp + -48], eax
    mov eax, [rbp + -48]
    mov [rbp + -8], eax
    jmp BB_1
BB_3:
    mov eax, 0
    mov [rbp + -56], eax
    mov eax, [rbp + -56]
