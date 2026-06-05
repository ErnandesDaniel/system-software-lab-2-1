extern row_buf
extern col_pipe
extern co
extern tv_buf
extern tv_id
extern tv_name
extern tv_n
extern pp_buf
extern pp_id
extern pp_surname
extern pp_name
extern pp_patr
extern pp_bday
extern pp_n
extern st_buf
extern st_pid
extern st_form
extern st_fac
extern st_course
extern st_n
extern sd_buf
extern sd_pid
extern sd_group
extern sd_start
extern sd_n
extern vv_buf
extern vv_tid
extern vv_pid
extern vv_mark
extern vv_date
extern vv_n
extern gp_buf
extern gp_pid
extern gp_group
extern gp_dept
extern gp_n
extern w
bits 64
default rel
section .text
global read_line

extern fgetc

    read_line:
    push rbp
    mov rbp, rsp
    sub rsp, 240
    mov [rbp + -16], rcx
    read_line_BB0:
    mov eax, 0
    mov [rbp + -32], rax
    mov eax, [rbp + -32]
    mov [rbp + -24], rax
    jmp read_line_BB1
    read_line_BB1:
    mov eax, 1
    mov [rbp + -40], rax
    mov eax, [rbp + -40]
    test eax, eax
    jne read_line_BB2
    jmp read_line_BB3
    read_line_BB2:
    mov rcx, [rbp + -16]
    sub rsp, 32
    xor eax, eax
    call fgetc
    add rsp, 32
    mov [rbp + -120], rax
    mov eax, [rbp + -120]
    mov [rbp + -8], rax
    mov eax, 1
    mov [rbp + -128], rax
    mov eax, [rbp + -128]
    neg eax
    mov [rbp + -136], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -136]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -144], rax
    mov eax, [rbp + -144]
    test eax, eax
    jne read_line_BB5
    jmp read_line_BB4
    read_line_BB3:
    read_line_BB4:
    mov eax, 10
    mov [rbp + -192], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -192]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -200], rax
    mov eax, [rbp + -200]
    test eax, eax
    jne read_line_BB9
    jmp read_line_BB8
    read_line_BB5:
    mov eax, 0
    mov [rbp + -64], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -64]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    test eax, eax
    jne read_line_BB7
    jmp read_line_BB6
    read_line_BB6:
    mov rax, [rel row_buf]
    mov [rbp + -80], rax
    mov rcx, [rbp + -80]
    mov edx, [rbp + -24]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -88], rax
    mov eax, 0
    mov [rbp + -96], rax
    mov rax, [rel row_buf]
    mov [rbp + -104], rax
    mov rcx, [rbp + -104]
    mov edx, [rbp + -24]
    mov r8d, [rbp + -96]
    mov [rcx + rdx], r8b
    mov eax, 0
    mov [rbp + -112], rax
    mov eax, [rbp + -112]
    leave
    ret
    read_line_BB7:
    mov eax, 1
    mov [rbp + -48], rax
    mov eax, [rbp + -48]
    neg eax
    mov [rbp + -56], rax
    mov eax, [rbp + -56]
    leave
    ret
    read_line_BB8:
    mov rax, [rel row_buf]
    mov [rbp + -208], rax
    mov rcx, [rbp + -208]
    mov edx, [rbp + -24]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -216], rax
    mov rax, [rel row_buf]
    mov [rbp + -224], rax
    mov rcx, [rbp + -224]
    mov edx, [rbp + -24]
    mov r8d, [rbp + -8]
    mov [rcx + rdx], r8b
    mov eax, 1
    mov [rbp + -232], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -232]
    add eax, ecx
    mov [rbp + -240], rax
    mov eax, [rbp + -240]
    mov [rbp + -24], rax
    jmp read_line_BB1
    read_line_BB9:
    mov rax, [rel row_buf]
    mov [rbp + -152], rax
    mov rcx, [rbp + -152]
    mov edx, [rbp + -24]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -160], rax
    mov eax, 0
    mov [rbp + -168], rax
    mov rax, [rel row_buf]
    mov [rbp + -176], rax
    mov rcx, [rbp + -176]
    mov edx, [rbp + -24]
    mov r8d, [rbp + -168]
    mov [rcx + rdx], r8b
    mov eax, 0
    mov [rbp + -184], rax
    mov eax, [rbp + -184]
    leave
    ret
