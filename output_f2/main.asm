bits 64
default rel
section .text

bits 64
default rel
section .text

global main
extern fclose
extern fgets
extern fopen
extern malloc
extern puts

main:
    push rbp
    mov rbp, rsp
    sub rsp, 128
BB_0:
    lea rax, [main_str_0]
    mov [rbp + -56], rax
    mov rax, [rbp + -56]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -64], eax
    mov eax, 64
    mov [rbp + -72], eax
    mov ecx, [rbp + -72]
    sub rsp, 32
    call malloc
    add rsp, 32
    mov [rbp + -80], rax
    mov rax, [rbp + -80]
    mov [rbp + -16], rax
    lea rax, [main_str_1]
    mov [rbp + -88], rax
    lea rax, [main_str_2]
    mov [rbp + -96], rax
    mov rax, [rbp + -88]
    mov rcx, rax
    mov rax, [rbp + -96]
    mov rdx, rax
    sub rsp, 32
    call fopen
    add rsp, 32
    mov [rbp + -104], rax
    mov rax, [rbp + -104]
    mov [rbp + -8], rax
    lea rax, [main_str_3]
    mov [rbp + -112], rax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -112]
    cmp eax, ebx
    setne al
    movzx eax, al
    mov [rbp + -120], eax
    mov eax, [rbp + -120]
    test eax, eax
    jne BB_1
    jmp BB_2
BB_1:
    mov eax, 64
    mov [rbp + -40], eax
    mov rax, [rbp + -16]
    mov rcx, rax
    mov edx, [rbp + -40]
    mov rax, [rbp + -8]
    mov r8, rax
    sub rsp, 32
    call fgets
    add rsp, 32
    mov [rbp + -48], rax
    jmp BB_3
BB_2:
    jmp BB_3
BB_3:
    mov rax, [rbp + -16]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -24], eax
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call fclose
    add rsp, 32
    mov [rbp + -32], eax

section .data
main_str_0 db 115, 116, 97, 114, 116, 0
main_str_1 db 108, 97, 98, 115, 45, 101, 120, 97, 109, 112, 108, 101, 115, 47, 115, 121, 115, 116, 101, 109, 45, 112, 114, 111, 103, 114, 97, 109, 109, 115, 47, 108, 97, 98, 45, 50, 47, 99, 115, 118, 45, 100, 97, 116, 97, 47, 112, 101, 111, 112, 108, 101, 46, 99, 115, 118, 0
main_str_2 db 114, 0
main_str_3 db 0, 0
