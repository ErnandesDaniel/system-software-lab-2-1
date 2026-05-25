extern buf
extern p_count
bits 64
default rel
section .text

bits 64
default rel
section .text

global load_people
extern fclose
extern feof
extern fgets
extern fopen

load_people:
    push rbp
    mov rbp, rsp
    sub rsp, 208
BB_0:
    lea rax, [load_people_str_0]
    mov [rbp + -40], rax
    lea rax, [load_people_str_1]
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
    lea rax, [load_people_str_2]
    mov [rbp + -64], rax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -64]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -72], eax
    mov eax, [rbp + -72]
    test eax, eax
    jne BB_1
    jmp BB_2
BB_1:
    mov eax, 1
    mov [rbp + -32], eax
    mov eax, [rbp + -32]
    leave
    ret
BB_2:
    jmp BB_3
BB_3:
    mov rax, [rel buf]
    mov [rbp + -160], rax
    mov eax, 256
    mov [rbp + -168], eax
    mov rax, [rbp + -160]
    mov rcx, rax
    mov edx, [rbp + -168]
    mov rax, [rbp + -8]
    mov r8, rax
    sub rsp, 32
    call fgets
    add rsp, 32
    mov [rbp + -176], rax
    mov eax, [rel p_count]
    mov [rbp + -184], eax
    mov eax, 0
    mov [rbp + -192], eax
    mov eax, [rbp + -192]
    mov [rel p_count], eax
    jmp BB_4
BB_4:
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call feof
    add rsp, 32
    mov [rbp + -80], eax
    mov eax, 0
    mov [rbp + -88], eax
    mov eax, [rbp + -80]
    mov ebx, [rbp + -88]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -96], eax
    mov eax, [rbp + -96]
    test eax, eax
    jne BB_5
    jmp BB_6
BB_5:
    mov rax, [rel buf]
    mov [rbp + -104], rax
    mov eax, 256
    mov [rbp + -112], eax
    mov rax, [rbp + -104]
    mov rcx, rax
    mov edx, [rbp + -112]
    mov rax, [rbp + -8]
    mov r8, rax
    sub rsp, 32
    call fgets
    add rsp, 32
    mov [rbp + -120], rax
    mov eax, [rel p_count]
    mov [rbp + -128], eax
    mov eax, [rel p_count]
    mov [rbp + -136], eax
    mov eax, 1
    mov [rbp + -144], eax
    mov eax, [rbp + -136]
    mov ebx, [rbp + -144]
    add eax, ebx
    mov [rbp + -152], eax
    mov eax, [rbp + -152]
    mov [rel p_count], eax
    jmp BB_4
BB_6:
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call fclose
    add rsp, 32
    mov [rbp + -16], eax
    mov eax, 0
    mov [rbp + -24], eax
    mov eax, [rbp + -24]
    leave
    ret

section .data
load_people_str_0 db 116, 101, 115, 116, 46, 99, 115, 118, 0
load_people_str_1 db 114, 0
load_people_str_2 db 0, 0
