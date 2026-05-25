extern CORO_COUNT
bits 64
default rel
section .text

bits 64
default rel
section .text

global co2
co2:
    push rbp
    mov rbp, rsp
    sub rsp, 32
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
    mov eax, 1
    mov rcx, [rbp + -8]
    mov [rcx], eax
    leave
    ret
