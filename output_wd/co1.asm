extern g
bits 64
default rel
section .text

bits 64
default rel
section .text

global co1
co1:
    push rbp
    mov rbp, rsp
    sub rsp, 64
    mov [rbp + -8], rcx
    mov eax, [rcx]
    cmp eax, 0
    je co_0
    cmp eax, 1
    je co_1
co_1:
BB_1:
    mov eax, 0
    mov [rbp + -16], eax
    mov eax, [rbp + -16]
    mov rcx, [rbp + -8]
    mov dword [rcx], -1
    mov dword [rcx + 16], eax
    leave
    ret
co_0:
BB_0:
    mov eax, [rel g]
    mov [rbp + -24], eax
    mov eax, [rel g]
    mov [rbp + -32], eax
    mov eax, 1
    mov [rbp + -40], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -40]
    add eax, ebx
    mov [rbp + -48], eax
    mov eax, [rbp + -48]
    mov [rel g], eax
    mov eax, 1
    mov rcx, [rbp + -8]
    mov [rcx], eax
    leave
    ret
