bits 64
default rel

section .text
global str_get_byte
str_get_byte:
    movzx eax, byte [rcx + rdx]
    ret

global str_set_byte
str_set_byte:
    mov [rcx + rdx], r8b
    ret
