extern x
bits 64
default rel
section .text

bits 64
default rel
section .text

global main
extern coro_init_nasm
extern puts
extern resume_coroutine_nasm

main:
    push rbp
    mov rbp, rsp
    sub rsp, 256
BB_0:
    sub rsp, 32
    call coro_init_nasm
    add rsp, 32
    mov [rbp + -216], eax
    mov eax, 0
    mov [rbp + -224], eax
    mov eax, [rbp + -224]
    mov [rbp + -8], eax
    mov eax, 0
    mov [rbp + -232], eax
    mov eax, [rbp + -232]
    mov [rbp + -24], eax
    mov eax, 0
    mov [rbp + -240], eax
    mov eax, [rbp + -240]
    mov [rbp + -16], eax
    jmp BB_1
BB_1:
    mov eax, 0
    mov [rbp + -56], eax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -56]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -64], eax
    mov eax, 0
    mov [rbp + -72], eax
    mov eax, [rbp + -24]
    mov ebx, [rbp + -72]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -80], eax
    mov eax, [rbp + -64]
    mov ebx, [rbp + -80]
    or eax, ebx
    mov [rbp + -88], eax
    mov eax, 0
    mov [rbp + -96], eax
    mov eax, [rbp + -16]
    mov ebx, [rbp + -96]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -104], eax
    mov eax, [rbp + -88]
    mov ebx, [rbp + -104]
    or eax, ebx
    mov [rbp + -112], eax
    mov eax, [rbp + -112]
    test eax, eax
    jne BB_2
    jmp BB_3
BB_2:
    mov eax, 0
    mov [rbp + -136], eax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -136]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -144], eax
    mov eax, [rbp + -144]
    test eax, eax
    jne BB_4
    jmp BB_5
BB_3:
    lea rax, [main_str_0]
    mov [rbp + -32], rax
    mov rax, [rbp + -32]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -40], eax
    mov eax, 0
    mov [rbp + -48], eax
    mov eax, [rbp + -48]
    leave
    ret
BB_4:
    mov eax, 0
    mov [rbp + -120], eax
    mov ecx, [rbp + -120]
    sub rsp, 32
    call resume_coroutine_nasm
    add rsp, 32
    mov [rbp + -128], eax
    mov eax, [rbp + -128]
    mov [rbp + -8], eax
    jmp BB_6
BB_5:
    jmp BB_6
BB_6:
    mov eax, 0
    mov [rbp + -168], eax
    mov eax, [rbp + -24]
    mov ebx, [rbp + -168]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -176], eax
    mov eax, [rbp + -176]
    test eax, eax
    jne BB_7
    jmp BB_8
BB_7:
    mov eax, 0
    mov [rbp + -152], eax
    mov ecx, [rbp + -152]
    sub rsp, 32
    call resume_coroutine_nasm
    add rsp, 32
    mov [rbp + -160], eax
    mov eax, [rbp + -160]
    mov [rbp + -24], eax
    jmp BB_9
BB_8:
    jmp BB_9
BB_9:
    mov eax, 0
    mov [rbp + -200], eax
    mov eax, [rbp + -16]
    mov ebx, [rbp + -200]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -208], eax
    mov eax, [rbp + -208]
    test eax, eax
    jne BB_10
    jmp BB_11
BB_10:
    mov eax, 0
    mov [rbp + -184], eax
    mov ecx, [rbp + -184]
    sub rsp, 32
    call resume_coroutine_nasm
    add rsp, 32
    mov [rbp + -192], eax
    mov eax, [rbp + -192]
    mov [rbp + -16], eax
    jmp BB_12
BB_11:
    jmp BB_12
BB_12:
    jmp BB_1

section .data
main_str_0 db 100, 111, 110, 101, 0
