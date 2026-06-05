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
global q1

extern print_str
extern printf
extern putchar
extern puts

    q1:
    push rbp
    mov rbp, rsp
    sub rsp, 320
    q1_BB0:
    lea rax, [q1_str_0]
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
    mov [rbp + -8], rax
    mov eax, 0
    mov [rbp + -56], rax
    mov eax, [rbp + -56]
    mov [rbp + -24], rax
    jmp q1_BB1
    q1_BB1:
    mov eax, [rel vv_n]
    mov [rbp + -64], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -64]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    test eax, eax
    jne q1_BB2
    jmp q1_BB3
    q1_BB2:
    mov rax, [rbp + -24]
    lea rdx, [rel vv_tid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -80], rcx
    mov eax, 3
    mov [rbp + -88], rax
    mov eax, [rbp + -80]
    mov ecx, [rbp + -88]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -96], rax
    mov eax, [rbp + -96]
    test eax, eax
    jne q1_BB5
    jmp q1_BB6
    q1_BB3:
    lea rax, [q1_str_1]
    mov [rbp + -312], rax
    mov rcx, [rbp + -312]
    mov edx, [rbp + -8]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -320], rax
    leave
    ret
    q1_BB4:
    mov eax, 1
    mov [rbp + -296], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -296]
    add eax, ecx
    mov [rbp + -304], rax
    mov eax, [rbp + -304]
    mov [rbp + -24], rax
    jmp q1_BB1
    q1_BB5:
    mov rax, [rbp + -24]
    lea rdx, [rel vv_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -104], rcx
    mov eax, 153285
    mov [rbp + -112], rax
    mov eax, [rbp + -104]
    mov ecx, [rbp + -112]
    cmp eax, ecx
    setg al
    movzx eax, al
    mov [rbp + -120], rax
    mov eax, [rbp + -120]
    mov [rbp + -128], rax
    jmp q1_BB7
    q1_BB6:
    mov eax, 0
    mov [rbp + -128], rax
    jmp q1_BB7
    q1_BB7:
    mov eax, [rbp + -128]
    test eax, eax
    jne q1_BB8
    jmp q1_BB4
    q1_BB8:
    mov eax, 0
    mov [rbp + -288], rax
    mov eax, [rbp + -288]
    mov [rbp + -16], rax
    jmp q1_BB9
    q1_BB9:
    mov eax, [rel tv_n]
    mov [rbp + -136], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -136]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -144], rax
    mov eax, [rbp + -144]
    test eax, eax
    jne q1_BB10
    jmp q1_BB11
    q1_BB10:
    mov rax, [rbp + -16]
    lea rdx, [rel tv_id]
    mov ecx, [rdx + rax*4]
    mov [rbp + -248], rcx
    mov eax, 3
    mov [rbp + -256], rax
    mov eax, [rbp + -248]
    mov ecx, [rbp + -256]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -264], rax
    mov eax, [rbp + -264]
    test eax, eax
    jne q1_BB13
    jmp q1_BB12
    q1_BB11:
    jmp q1_BB4
    q1_BB12:
    mov eax, 1
    mov [rbp + -272], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -272]
    add eax, ecx
    mov [rbp + -280], rax
    mov eax, [rbp + -280]
    mov [rbp + -16], rax
    jmp q1_BB9
    q1_BB13:
    mov rax, [rel tv_buf]
    mov [rbp + -152], rax
    mov rax, [rbp + -16]
    lea rdx, [rel tv_name]
    mov ecx, [rdx + rax*4]
    mov [rbp + -160], rcx
    mov rcx, [rbp + -152]
    mov edx, [rbp + -160]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    mov eax, 44
    mov [rbp + -168], rax
    mov ecx, [rbp + -168]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -176], rax
    mov eax, 32
    mov [rbp + -184], rax
    mov ecx, [rbp + -184]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -192], rax
    mov rax, [rel vv_buf]
    mov [rbp + -200], rax
    mov rax, [rbp + -24]
    lea rdx, [rel vv_date]
    mov ecx, [rdx + rax*4]
    mov [rbp + -208], rcx
    mov rcx, [rbp + -200]
    mov edx, [rbp + -208]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    mov eax, 10
    mov [rbp + -216], rax
    mov ecx, [rbp + -216]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -224], rax
    mov eax, 1
    mov [rbp + -232], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -232]
    add eax, ecx
    mov [rbp + -240], rax
    mov eax, [rbp + -240]
    mov [rbp + -8], rax
    jmp q1_BB12

section .data
q1_str_0 db 61, 61, 61, 32, 81, 49, 58, 32, 73, 78, 78, 69, 82, 32, 74, 79, 73, 78, 32, 61, 61, 61, 0
q1_str_1 db 70, 111, 117, 110, 100, 58, 32, 37, 100, 10, 0
