bits 64
default rel
section .text

global main
main:
    push rbp
    mov rbp, rsp
    sub rsp, 144
BB_0:
    mov eax, 1
    mov [rbp + -32], eax
    mov eax, [rbp + -32]
    mov [rbp + -16], eax
    jmp BB_1
BB_1:
    mov eax, 5
    mov [rbp + -40], eax
    mov eax, [rbp + -16]
    mov eax, [rbp + -40]
    mov ebx, eax
    cmp eax, ebx
    setl al
    movzx eax, al
    mov [rbp + -48], eax
    mov eax, [rbp + -48]
    test eax, eax
    jne BB_2
    jmp BB_3
BB_2:
    mov eax, 1
    mov [rbp + -56], eax
    mov eax, [rbp + -16]
    mov eax, [rbp + -56]
    mov ebx, eax
    add eax, ebx
    mov [rbp + -64], eax
    mov eax, [rbp + -64]
    mov [rbp + -16], eax
    jmp BB_1
BB_3:
    lea rax, [str_0]
    mov [rbp + -72], rax
    mov rax, [rbp + -72]
    mov [rbp + -24], rax
    mov eax, 1
    mov [rbp + -80], eax
    mov eax, 5
    mov [rbp + -88], eax
    mov eax, 2
    mov [rbp + -96], eax
    mov eax, [rbp + -88]
    mov eax, [rbp + -96]
    mov ebx, eax
    imul eax, ebx
    mov [rbp + -104], eax
    mov eax, [rbp + -80]
    mov eax, [rbp + -104]
    mov ebx, eax
    add eax, ebx
    mov [rbp + -112], eax
    mov eax, 4
    mov [rbp + -120], eax
    mov eax, [rbp + -112]
    mov eax, [rbp + -120]
    mov ebx, eax
    add eax, ebx
    mov [rbp + -128], eax
    mov eax, [rbp + -128]
    mov [rbp + -8], eax
    mov eax, 0
    mov [rbp + -136], eax
    mov eax, [rbp + -136]

section .data
str_0 db 102, 97, 115, 100, 102, 97, 100, 115, 102, 0
