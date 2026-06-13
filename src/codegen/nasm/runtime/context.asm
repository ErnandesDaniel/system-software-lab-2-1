bits 64
global switch_context_nasm

section .text

switch_context_nasm:
    ; RDI = old_ctx (save current RSP here)
    ; RSI = new_ctx (load new RSP from here)

    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15

    mov [rdi], rsp

    mov rsp, [rsi]

    pop r15
    pop r14
    pop r13
    pop r12
    pop rbx
    pop rbp

    ret
