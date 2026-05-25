bits 64
default rel
section .text

bits 64
default rel
section .text

global main
extern fclose
extern fgetc
extern fopen
extern puts

main:
    push rbp
    mov rbp, rsp
    sub rsp, 144
BB_0:
    lea rax, [main_str_0]
    mov [rbp + -24], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -32], eax
    lea rax, [main_str_1]
    mov [rbp + -40], rax
    lea rax, [main_str_2]
    mov [rbp + -48], rax
    mov rax, [rbp + -40]
    mov rcx, rax
    mov rax, [rbp + -48]
    mov rdx, rax
    sub rsp, 32
    call fopen
    add rsp, 32
    mov [rbp + -56], rax
    mov rax, [rbp + -56]
    mov [rbp + -8], rax
    lea rax, [main_str_3]
    mov [rbp + -64], rax
    mov rax, [rbp + -64]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -72], eax
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call fgetc
    add rsp, 32
    mov [rbp + -80], eax
    mov eax, [rbp + -80]
    mov [rbp + -16], eax
    lea rax, [main_str_4]
    mov [rbp + -88], rax
    mov rax, [rbp + -88]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -96], eax
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call fclose
    add rsp, 32
    mov [rbp + -104], eax
    lea rax, [main_str_5]
    mov [rbp + -112], rax
    mov rax, [rbp + -112]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -120], eax
    mov eax, 0
    mov [rbp + -128], eax
    mov eax, [rbp + -128]
    leave
    ret

section .data
main_str_0 db 115, 116, 97, 114, 116, 0
main_str_1 db 108, 97, 98, 115, 45, 101, 120, 97, 109, 112, 108, 101, 115, 47, 115, 121, 115, 116, 101, 109, 45, 112, 114, 111, 103, 114, 97, 109, 109, 115, 47, 108, 97, 98, 45, 50, 47, 99, 115, 118, 45, 100, 97, 116, 97, 47, 112, 101, 111, 112, 108, 101, 46, 99, 115, 118, 0
main_str_2 db 114, 0
main_str_3 db 111, 112, 101, 110, 101, 100, 0
main_str_4 db 114, 101, 97, 100, 32, 111, 110, 101, 32, 99, 104, 97, 114, 0
main_str_5 db 100, 111, 110, 101, 0
