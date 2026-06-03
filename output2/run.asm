bits 64
default rel
section .text

global run

extern resume_coroutine

run:
    push rbp
    mov rbp, rsp
    sub rsp, 160
run_BB0:
    mov eax, 0
    mov [rbp + -24], eax
    mov eax, [rbp + -24]
    mov [rbp + -8], eax
    mov eax, 0
    mov [rbp + -32], eax
    mov eax, [rbp + -32]
    mov [rbp + -16], eax
    jmp run_BB1
run_BB1:
    mov eax, 0
    mov [rbp + -40], eax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -40]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -48], eax
    mov eax, 0
    mov [rbp + -56], eax
    mov eax, [rbp + -16]
    mov ebx, [rbp + -56]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -64], eax
    mov eax, [rbp + -48]
    mov ebx, [rbp + -64]
    or eax, ebx
    mov [rbp + -72], eax
    mov eax, [rbp + -72]
    test eax, eax
    jne run_BB2
    jmp run_BB3
run_BB2:
    mov eax, 0
    mov [rbp + -96], eax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -96]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -104], eax
    mov eax, [rbp + -104]
    test eax, eax
    jne run_BB5
    jmp run_BB4
run_BB3:
    mov eax, 0
    mov [rbp + -144], eax
    mov eax, [rbp + -144]
    leave
    ret
run_BB4:
    mov eax, 0
    mov [rbp + -128], eax
    mov eax, [rbp + -16]
    mov ebx, [rbp + -128]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -136], eax
    mov eax, [rbp + -136]
    test eax, eax
    jne run_BB7
    jmp run_BB6
run_BB5:
    mov eax, 0
    mov [rbp + -80], eax
    mov ecx, [rbp + -80]
    sub rsp, 32
    call resume_coroutine
    add rsp, 32
    mov [rbp + -88], eax
    mov eax, [rbp + -88]
    mov [rbp + -8], eax
    jmp run_BB4
run_BB6:
    jmp run_BB1
run_BB7:
    mov eax, 1
    mov [rbp + -112], eax
    mov ecx, [rbp + -112]
    sub rsp, 32
    call resume_coroutine
    add rsp, 32
    mov [rbp + -120], eax
    mov eax, [rbp + -120]
    mov [rbp + -16], eax
    jmp run_BB6
