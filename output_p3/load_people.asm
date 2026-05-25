extern buf
bits 64
default rel
section .text

bits 64
default rel
section .text

global load_people
extern fclose
extern fgets
extern fopen

load_people:
    push rbp
    mov rbp, rsp
    sub rsp, 112
BB_0:
    lea rax, [load_people_str_0]
    mov [rbp + -64], rax
    lea rax, [load_people_str_1]
    mov [rbp + -72], rax
    mov rax, [rbp + -64]
    mov rcx, rax
    mov rax, [rbp + -72]
    mov rdx, rax
    sub rsp, 32
    call fopen
    add rsp, 32
    mov [rbp + -80], rax
    mov rax, [rbp + -80]
    mov [rbp + -8], rax
    lea rax, [load_people_str_2]
    mov [rbp + -88], rax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -88]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -96], eax
    mov eax, [rbp + -96]
    test eax, eax
    jne BB_1
    jmp BB_2
BB_1:
    mov eax, 1
    mov [rbp + -56], eax
    mov eax, [rbp + -56]
    leave
    ret
BB_2:
    jmp BB_3
BB_3:
    mov rax, [rel buf]
    mov [rbp + -16], rax
    mov eax, 256
    mov [rbp + -24], eax
    mov rax, [rbp + -16]
    mov rcx, rax
    mov edx, [rbp + -24]
    mov rax, [rbp + -8]
    mov r8, rax
    sub rsp, 32
    call fgets
    add rsp, 32
    mov [rbp + -32], rax
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call fclose
    add rsp, 32
    mov [rbp + -40], eax
    mov eax, 0
    mov [rbp + -48], eax
    mov eax, [rbp + -48]
    leave
    ret

section .data
load_people_str_0 db 116, 101, 115, 116, 46, 99, 115, 118, 0
load_people_str_1 db 114, 0
load_people_str_2 db 0, 0
