bits 64
default rel
section .text

bits 64
default rel
section .text

global main
extern puts

main:
    push rbp
    mov rbp, rsp
    sub rsp, 176
BB_0:
    mov eax, 0
    mov [rbp + -144], eax
    mov eax, [rbp + -144]
    mov [rbp + -24], eax
    mov eax, 1
    mov [rbp + -152], eax
    mov eax, [rbp + -152]
    mov [rbp + -16], eax
    mov eax, 0
    mov [rbp + -160], eax
    mov eax, [rbp + -160]
    mov [rbp + -8], eax
    jmp BB_1
BB_1:
    mov eax, 0
    mov [rbp + -56], eax
    mov eax, [rbp + -24]
    mov ebx, [rbp + -56]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -64], eax
    mov eax, 0
    mov [rbp + -72], eax
    mov eax, [rbp + -16]
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
    mov eax, [rbp + -8]
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
    mov eax, 1
    mov [rbp + -120], eax
    mov eax, [rbp + -120]
    mov [rbp + -24], eax
    mov eax, 1
    mov [rbp + -128], eax
    mov eax, [rbp + -128]
    mov [rbp + -16], eax
    mov eax, 1
    mov [rbp + -136], eax
    mov eax, [rbp + -136]
    mov [rbp + -8], eax
    jmp BB_1
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

section .data
main_str_0 db 100, 111, 110, 101, 0
