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
global q2

extern printf
extern putchar
extern puts
extern str_eq_lit

    q2:
    push rbp
    mov rbp, rsp
    sub rsp, 320
    q2_BB0:
    lea rax, [q2_str_0]
    mov [rbp + -32], rax
    mov rcx, [rbp + -32]
    sub rsp, 32
    xor eax, eax
    call puts
    add rsp, 32
    mov [rbp + -40], rax
    mov eax, 0
    mov [rbp + -48], rax
    mov eax, [rbp + -48]
    mov [rbp + -16], rax
    mov eax, 0
    mov [rbp + -56], rax
    mov eax, [rbp + -56]
    mov [rbp + -8], rax
    jmp q2_BB1
    q2_BB1:
    mov eax, [rel st_n]
    mov [rbp + -64], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -64]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    test eax, eax
    jne q2_BB2
    jmp q2_BB3
    q2_BB2:
    mov rax, [rbp + -8]
    lea rdx, [rel st_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -264], rcx
    mov eax, 163276
    mov [rbp + -272], rax
    mov eax, [rbp + -264]
    mov ecx, [rbp + -272]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -280], rax
    mov eax, [rbp + -280]
    test eax, eax
    jne q2_BB5
    jmp q2_BB4
    q2_BB3:
    lea rax, [q2_str_1]
    mov [rbp + -304], rax
    mov rcx, [rbp + -304]
    mov edx, [rbp + -16]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -312], rax
    leave
    ret
    q2_BB4:
    mov eax, 1
    mov [rbp + -288], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -288]
    add eax, ecx
    mov [rbp + -296], rax
    mov eax, [rbp + -296]
    mov [rbp + -8], rax
    jmp q2_BB1
    q2_BB5:
    mov eax, 0
    mov [rbp + -256], rax
    mov eax, [rbp + -256]
    mov [rbp + -24], rax
    jmp q2_BB6
    q2_BB6:
    mov eax, [rel sd_n]
    mov [rbp + -80], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -80]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -88], rax
    mov eax, [rbp + -88]
    test eax, eax
    jne q2_BB7
    jmp q2_BB8
    q2_BB7:
    mov rax, [rbp + -24]
    lea rdx, [rel sd_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -96], rcx
    mov rax, [rbp + -8]
    lea rdx, [rel st_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -104], rcx
    mov eax, [rbp + -96]
    mov ecx, [rbp + -104]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -112], rax
    mov eax, [rbp + -112]
    test eax, eax
    jne q2_BB10
    jmp q2_BB11
    q2_BB8:
    jmp q2_BB4
    q2_BB9:
    mov eax, 1
    mov [rbp + -240], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -240]
    add eax, ecx
    mov [rbp + -248], rax
    mov eax, [rbp + -248]
    mov [rbp + -24], rax
    jmp q2_BB6
    q2_BB10:
    mov rax, [rel sd_buf]
    mov [rbp + -120], rax
    mov rax, [rbp + -24]
    lea rdx, [rel sd_start]
    mov ecx, [rdx + rax*4]
    mov [rbp + -128], rcx
    lea rax, [q2_str_2]
    mov [rbp + -136], rax
    mov rcx, [rbp + -120]
    mov edx, [rbp + -128]
    mov r8, [rbp + -136]
    sub rsp, 32
    xor eax, eax
    call str_eq_lit
    add rsp, 32
    mov [rbp + -144], rax
    mov eax, 0
    mov [rbp + -152], rax
    mov eax, [rbp + -144]
    mov ecx, [rbp + -152]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -160], rax
    mov eax, [rbp + -160]
    mov [rbp + -168], rax
    jmp q2_BB12
    q2_BB11:
    mov eax, 0
    mov [rbp + -168], rax
    jmp q2_BB12
    q2_BB12:
    mov eax, [rbp + -168]
    test eax, eax
    jne q2_BB13
    jmp q2_BB9
    q2_BB13:
    lea rax, [q2_str_3]
    mov [rbp + -176], rax
    mov rax, [rbp + -8]
    lea rdx, [rel st_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -184], rcx
    mov rax, [rbp + -24]
    lea rdx, [rel sd_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -192], rcx
    mov rcx, [rbp + -176]
    mov edx, [rbp + -184]
    mov r8d, [rbp + -192]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -200], rax
    mov eax, 10
    mov [rbp + -208], rax
    mov ecx, [rbp + -208]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -216], rax
    mov eax, 1
    mov [rbp + -224], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -224]
    add eax, ecx
    mov [rbp + -232], rax
    mov eax, [rbp + -232]
    mov [rbp + -16], rax
    jmp q2_BB9

section .data
q2_str_0 db 61, 61, 61, 32, 81, 50, 58, 32, 76, 69, 70, 84, 32, 74, 79, 73, 78, 32, 61, 61, 61, 0
q2_str_1 db 70, 111, 117, 110, 100, 58, 32, 37, 100, 10, 0
q2_str_2 db 50, 48, 48, 56, 45, 48, 57, 45, 48, 49, 0
q2_str_3 db 37, 100, 44, 32, 79, 75, 53, 48, 48, 44, 32, 37, 100, 0
