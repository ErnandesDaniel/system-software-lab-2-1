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
    sub rsp, 176
BB_0:
    sub rsp, 32
    call coro_init_nasm
    add rsp, 32
    mov [rbp + -152], eax
    mov eax, 0
    mov [rbp + -160], eax
    mov eax, [rbp + -160]
    mov [rbp + -8], eax
    mov eax, 0
    mov [rbp + -168], eax
    mov eax, [rbp + -168]
    mov [rbp + -16], eax
    jmp BB_1
BB_1:
    mov eax, 0
    mov [rbp + -48], eax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -48]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -56], eax
    mov eax, 0
    mov [rbp + -64], eax
    mov eax, [rbp + -16]
    mov ebx, [rbp + -64]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -72], eax
    mov eax, [rbp + -56]
    mov ebx, [rbp + -72]
    or eax, ebx
    mov [rbp + -80], eax
    mov eax, [rbp + -80]
    test eax, eax
    jne BB_2
    jmp BB_3
BB_2:
    mov eax, 0
    mov [rbp + -104], eax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -104]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -112], eax
    mov eax, [rbp + -112]
    test eax, eax
    jne BB_4
    jmp BB_5
BB_3:
    lea rax, [main_str_0]
    mov [rbp + -24], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -32], eax
    mov eax, 0
    mov [rbp + -40], eax
    mov eax, [rbp + -40]
    leave
    ret
BB_4:
    mov eax, 0
    mov [rbp + -88], eax
    mov ecx, [rbp + -88]
    sub rsp, 32
    call resume_coroutine_nasm
    add rsp, 32
    mov [rbp + -96], eax
    mov eax, [rbp + -96]
    mov [rbp + -8], eax
    jmp BB_6
BB_5:
    jmp BB_6
BB_6:
    mov eax, 0
    mov [rbp + -136], eax
    mov eax, [rbp + -16]
    mov ebx, [rbp + -136]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -144], eax
    mov eax, [rbp + -144]
    test eax, eax
    jne BB_7
    jmp BB_8
BB_7:
    mov eax, 0
    mov [rbp + -120], eax
    mov ecx, [rbp + -120]
    sub rsp, 32
    call resume_coroutine_nasm
    add rsp, 32
    mov [rbp + -128], eax
    mov eax, [rbp + -128]
    mov [rbp + -16], eax
    jmp BB_9
BB_8:
    jmp BB_9
BB_9:
    jmp BB_1

section .data
main_str_0 db 100, 111, 110, 101, 0
