extern buf
bits 64
default rel
section .text

bits 64
default rel
section .text

global read_line
read_line:
    push rbp
    mov rbp, rsp
    sub rsp, 48
    mov [rbp + -24], rcx
    mov [rbp + -32], rdx
BB_0:
    mov eax, 0
    mov [rbp + -40], eax
    mov eax, [rbp + -40]
    leave
    ret
