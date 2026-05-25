extern buf
bits 64
default rel
section .text

bits 64
default rel
section .text

global read_line
extern fgetc
extern str_set_byte

read_line:
    push rbp
    mov rbp, rsp
    sub rsp, 352
    mov [rbp + -48], rcx
    mov [rbp + -56], rdx
BB_0:
    mov eax, 0
    mov [rbp + -336], eax
    mov eax, [rbp + -336]
    mov [rbp + -8], eax
    jmp BB_1
BB_1:
    mov eax, 1
    mov [rbp + -64], eax
    mov eax, 1
    mov [rbp + -72], eax
    mov eax, [rbp + -64]
    mov ebx, [rbp + -72]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -80], eax
    mov eax, [rbp + -80]
    test eax, eax
    jne BB_2
    jmp BB_3
BB_2:
    mov rax, [rbp + -56]
    mov rcx, rax
    sub rsp, 32
    call fgetc
    add rsp, 32
    mov [rbp + -136], eax
    mov eax, [rbp + -136]
    mov [rbp + -32], eax
    mov eax, 0
    mov [rbp + -144], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -144]
    cmp eax, ebx
    setl al
    movzx eax, al
    mov [rbp + -152], eax
    mov eax, [rbp + -152]
    test eax, eax
    jne BB_4
    jmp BB_5
BB_3:
    leave
    ret
BB_4:
    mov eax, 0
    mov [rbp + -96], eax
    mov rax, [rbp + -48]
    mov rcx, rax
    mov edx, [rbp + -8]
    mov r8d, [rbp + -96]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -104], eax
    mov eax, 0
    mov [rbp + -112], eax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -112]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -120], eax
    mov eax, [rbp + -120]
    test eax, eax
    jne BB_7
    jmp BB_8
BB_5:
    jmp BB_6
BB_6:
    mov eax, 10
    mov [rbp + -184], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -184]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -192], eax
    mov eax, [rbp + -192]
    test eax, eax
    jne BB_10
    jmp BB_11
BB_7:
    mov eax, 1
    mov [rbp + -88], eax
    mov eax, [rbp + -88]
    leave
    ret
BB_8:
    jmp BB_9
BB_9:
    mov eax, 0
    mov [rbp + -128], eax
    mov eax, [rbp + -128]
    leave
    ret
BB_10:
    mov eax, 0
    mov [rbp + -160], eax
    mov rax, [rbp + -48]
    mov rcx, rax
    mov edx, [rbp + -8]
    mov r8d, [rbp + -160]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -168], eax
    mov eax, 0
    mov [rbp + -176], eax
    mov eax, [rbp + -176]
    leave
    ret
BB_11:
    jmp BB_12
BB_12:
    mov eax, 13
    mov [rbp + -296], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -296]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -304], eax
    mov eax, [rbp + -304]
    test eax, eax
    jne BB_13
    jmp BB_14
BB_13:
    mov rax, [rbp + -56]
    mov rcx, rax
    sub rsp, 32
    call fgetc
    add rsp, 32
    mov [rbp + -224], eax
    mov eax, [rbp + -224]
    mov [rbp + -24], eax
    mov eax, 10
    mov [rbp + -232], eax
    mov eax, [rbp + -24]
    mov ebx, [rbp + -232]
    cmp eax, ebx
    setne al
    movzx eax, al
    mov [rbp + -240], eax
    mov eax, 0
    mov [rbp + -248], eax
    mov eax, [rbp + -24]
    mov ebx, [rbp + -248]
    cmp eax, ebx
    setge al
    movzx eax, al
    mov [rbp + -256], eax
    mov eax, [rbp + -240]
    mov ebx, [rbp + -256]
    and eax, ebx
    mov [rbp + -264], eax
    mov eax, [rbp + -264]
    test eax, eax
    jne BB_16
    jmp BB_17
BB_14:
    jmp BB_15
BB_15:
    mov rax, [rbp + -48]
    mov rcx, rax
    mov edx, [rbp + -8]
    mov r8d, [rbp + -32]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -312], eax
    mov eax, 1
    mov [rbp + -320], eax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -320]
    add eax, ebx
    mov [rbp + -328], eax
    mov eax, [rbp + -328]
    mov [rbp + -8], eax
    jmp BB_1
BB_16:
    mov rax, [rbp + -48]
    mov rcx, rax
    mov edx, [rbp + -8]
    mov r8d, [rbp + -32]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -200], eax
    mov eax, 1
    mov [rbp + -208], eax
    mov eax, [rbp + -8]
    mov ebx, [rbp + -208]
    add eax, ebx
    mov [rbp + -216], eax
    mov eax, [rbp + -216]
    mov [rbp + -8], eax
    mov eax, [rbp + -24]
    mov [rbp + -32], eax
    jmp BB_18
BB_17:
    jmp BB_18
BB_18:
    mov eax, 0
    mov [rbp + -272], eax
    mov rax, [rbp + -48]
    mov rcx, rax
    mov edx, [rbp + -8]
    mov r8d, [rbp + -272]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -280], eax
    mov eax, 0
    mov [rbp + -288], eax
    mov eax, [rbp + -288]
    leave
    ret
