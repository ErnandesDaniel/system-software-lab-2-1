bits 64
default rel
section .text

global add
add:
    push rbp
    mov rbp, rsp
    sub rsp, 32
BB_0:
    mov eax, [rbp + -16]
    mov ebx, [rbp + -8]
    add eax, ebx
    mov [rbp + -24], eax
    mov eax, [rbp + -24]
; Очистка стека и возврат
    leave       ; эквивалент: mov rsp, rbp; pop rbp
    ret         ; возвращаем eax как результат
