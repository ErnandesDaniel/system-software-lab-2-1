extern CORO_COUNT
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
    sub rsp, 224
BB_0:
    mov eax, [rel CORO_COUNT]
    mov [rbp + -184], eax
    mov eax, 0
    mov [rbp + -192], eax
    mov eax, [rbp + -192]
    mov [rel CORO_COUNT], eax
    sub rsp, 32
    call coro_init_nasm
    add rsp, 32
    mov [rbp + -200], eax
    mov eax, 0
    mov [rbp + -208], eax
    mov eax, [rbp + -208]
    mov [rbp + -16], eax
    mov eax, 0
    mov [rbp + -216], eax
    mov eax, [rbp + -216]
    mov [rbp + -8], eax
    jmp BB_1
BB_1:
    mov eax, 0
    mov [rbp + -48], eax
    mov eax, [rbp + -16]
    mov ebx, [rbp + -48]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -56], eax
    mov eax, 0
    mov [rbp + -64], eax
    mov eax, [rbp + -8]
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
    mov [rbp + -120], eax
    mov eax, [rbp + -16]
    mov ebx, [rbp + -120]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -128], eax
    mov eax, [rbp + -128]
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
    mov eax, [rel CORO_COUNT]
    mov [rbp + -88], eax
    mov eax, 0
    mov [rbp + -96], eax
    mov eax, [rbp + -88]
    mov ebx, [rbp + -96]
    add eax, ebx
    mov [rbp + -104], eax
    mov ecx, [rbp + -104]
    sub rsp, 32
    call resume_coroutine_nasm
    add rsp, 32
    mov [rbp + -112], eax
    mov eax, [rbp + -112]
    mov [rbp + -16], eax
    jmp BB_6
BB_5:
    jmp BB_6
BB_6:
    mov eax, 0
    mov [rbp + -168], eax
    mov eax, [rbp + -8]
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
    mov eax, [rel CORO_COUNT]
    mov [rbp + -136], eax
    mov eax, 1
    mov [rbp + -144], eax
    mov eax, [rbp + -136]
    mov ebx, [rbp + -144]
    add eax, ebx
    mov [rbp + -152], eax
    mov ecx, [rbp + -152]
    sub rsp, 32
    call resume_coroutine_nasm
    add rsp, 32
    mov [rbp + -160], eax
    mov eax, [rbp + -160]
    mov [rbp + -8], eax
    jmp BB_9
BB_8:
    jmp BB_9
BB_9:
    jmp BB_1

section .data
main_str_0 db 100, 111, 110, 101, 0
