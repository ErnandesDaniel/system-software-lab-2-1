bits 64
default rel
section .text

global print_once

extern putchar

print_once:
    push rbp
    mov rbp, rsp
    sub rsp, 48
    mov [rbp + -8], rcx
    mov eax, [rcx]
    cmp eax, 0
    je print_once_co_0
    cmp eax, 1
    je print_once_co_1
print_once_co_0:
print_once_BB0:
    jmp print_once_BB1
print_once_BB1:
    mov eax, 1
    mov [rbp + -16], eax
    mov eax, [rbp + -16]
    test eax, eax
    jne print_once_BB2
    jmp print_once_BB3
print_once_BB2:
    mov eax, 49
    mov [rbp + -24], eax
    mov ecx, [rbp + -24]
    sub rsp, 32
    call putchar
    add rsp, 32
    mov [rbp + -32], eax
    mov eax, 1
    mov rcx, [rbp + -8]
    mov [rcx], eax
    leave
    ret
print_once_co_1:
print_once_BB4:
    jmp print_once_BB1
print_once_BB3:
    mov eax, 0
    mov [rbp + -40], eax
    mov eax, [rbp + -40]
    mov rcx, [rbp + -8]
    mov dword [rcx], -1
    mov dword [rcx + 16], eax
    leave
    ret
