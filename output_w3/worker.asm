extern x
bits 64
default rel
section .text

bits 64
default rel
section .text

global worker
worker:
    push rbp
    mov rbp, rsp
    sub rsp, 96
    mov [rbp + -8], rcx
    mov eax, [rcx]
    cmp eax, 0
    je co_0
    cmp eax, 1
    je co_1
co_1:
BB_1:
    mov eax, [rel x]
    mov [rbp + -16], eax
    mov eax, [rel x]
    mov [rbp + -24], eax
    mov eax, 1
    mov [rbp + -32], eax
    mov eax, [rbp + -24]
    mov ebx, [rbp + -32]
    add eax, ebx
    mov [rbp + -40], eax
    mov eax, [rbp + -40]
    mov [rel x], eax
    mov eax, 0
    mov [rbp + -48], eax
    mov eax, [rbp + -48]
    mov rcx, [rbp + -8]
    mov dword [rcx], -1
    mov dword [rcx + 16], eax
    leave
    ret
co_0:
BB_0:
    mov eax, [rel x]
    mov [rbp + -56], eax
    mov eax, [rel x]
    mov [rbp + -64], eax
    mov eax, 1
    mov [rbp + -72], eax
    mov eax, [rbp + -64]
    mov ebx, [rbp + -72]
    add eax, ebx
    mov [rbp + -80], eax
    mov eax, [rbp + -80]
    mov [rel x], eax
    mov eax, 1
    mov rcx, [rbp + -8]
    mov [rcx], eax
    leave
    ret
