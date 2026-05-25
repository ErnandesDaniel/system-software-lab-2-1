extern buf
bits 64
default rel
section .text

bits 64
default rel
section .text

global main
extern fclose
extern fopen
extern puts
extern read_line

main:
    push rbp
    mov rbp, rsp
    sub rsp, 176
BB_0:
    lea rax, [main_str_0]
    mov [rbp + -96], rax
    mov rax, [rbp + -96]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -104], eax
    lea rax, [main_str_1]
    mov [rbp + -112], rax
    lea rax, [main_str_2]
    mov [rbp + -120], rax
    mov rax, [rbp + -112]
    mov rcx, rax
    mov rax, [rbp + -120]
    mov rdx, rax
    sub rsp, 32
    call fopen
    add rsp, 32
    mov [rbp + -128], rax
    mov rax, [rbp + -128]
    mov [rbp + -8], rax
    lea rax, [main_str_3]
    mov [rbp + -136], rax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -136]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -144], eax
    mov eax, [rbp + -144]
    test eax, eax
    jne BB_1
    jmp BB_2
BB_1:
    lea rax, [main_str_4]
    mov [rbp + -64], rax
    mov rax, [rbp + -64]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -72], eax
    jmp BB_3
BB_2:
    lea rax, [main_str_5]
    mov [rbp + -80], rax
    mov rax, [rbp + -80]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -88], eax
    jmp BB_3
BB_3:
    lea rax, [main_str_6]
    mov [rbp + -160], rax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -160]
    cmp eax, ebx
    setne al
    movzx eax, al
    mov [rbp + -168], eax
    mov eax, [rbp + -168]
    test eax, eax
    jne BB_4
    jmp BB_5
BB_4:
    mov rax, [rel buf]
    mov [rbp + -152], rax
    mov rax, [rbp + -152]
    mov rcx, rax
    mov rax, [rbp + -8]
    mov rdx, rax
    sub rsp, 32
    call read_line
    add rsp, 32
    jmp BB_6
BB_5:
    jmp BB_6
BB_6:
    mov rax, [rel buf]
    mov [rbp + -16], rax
    mov rax, [rbp + -16]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -24], eax
    mov rax, [rel buf]
    mov [rbp + -32], rax
    mov rax, [rbp + -32]
    mov rcx, rax
    mov rax, [rbp + -8]
    mov rdx, rax
    sub rsp, 32
    call read_line
    add rsp, 32
    mov rax, [rel buf]
    mov [rbp + -40], rax
    mov rax, [rbp + -40]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -48], eax
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call fclose
    add rsp, 32
    mov [rbp + -56], eax

section .data
main_str_0 db 111, 112, 101, 110, 105, 110, 103, 0
main_str_1 db 108, 97, 98, 115, 45, 101, 120, 97, 109, 112, 108, 101, 115, 47, 115, 121, 115, 116, 101, 109, 45, 112, 114, 111, 103, 114, 97, 109, 109, 115, 47, 108, 97, 98, 45, 50, 47, 99, 115, 118, 45, 100, 97, 116, 97, 47, 112, 101, 111, 112, 108, 101, 46, 99, 115, 118, 0
main_str_2 db 114, 0
main_str_3 db 0, 0
main_str_4 db 70, 65, 73, 76, 0
main_str_5 db 79, 75, 0
main_str_6 db 0, 0
