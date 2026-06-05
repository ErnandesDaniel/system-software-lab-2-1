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
global group_avg

extern str_eq_lit

    group_avg:
    push rbp
    mov rbp, rsp
    sub rsp, 320
    mov [rbp + -24], rcx
    group_avg_BB0:
    mov eax, 0
    mov [rbp + -56], rax
    mov eax, [rbp + -56]
    mov [rbp + -40], rax
    mov eax, 0
    mov [rbp + -64], rax
    mov eax, [rbp + -64]
    mov [rbp + -16], rax
    mov eax, 0
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    mov [rbp + -32], rax
    jmp group_avg_BB1
    group_avg_BB1:
    mov eax, [rel sd_n]
    mov [rbp + -80], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -80]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -88], rax
    mov eax, [rbp + -88]
    test eax, eax
    jne group_avg_BB2
    jmp group_avg_BB3
    group_avg_BB2:
    mov rax, [rel sd_buf]
    mov [rbp + -224], rax
    mov rax, [rbp + -32]
    lea rdx, [rel sd_group]
    mov ecx, [rdx + rax*4]
    mov [rbp + -232], rcx
    mov rcx, [rbp + -224]
    mov edx, [rbp + -232]
    mov r8, [rbp + -24]
    sub rsp, 32
    xor eax, eax
    call str_eq_lit
    add rsp, 32
    mov [rbp + -240], rax
    mov eax, 0
    mov [rbp + -248], rax
    mov eax, [rbp + -240]
    mov ecx, [rbp + -248]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -256], rax
    mov eax, [rbp + -256]
    test eax, eax
    jne group_avg_BB5
    jmp group_avg_BB4
    group_avg_BB3:
    mov eax, 0
    mov [rbp + -288], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -288]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -296], rax
    mov eax, [rbp + -296]
    test eax, eax
    jne group_avg_BB15
    jmp group_avg_BB14
    group_avg_BB4:
    mov eax, 1
    mov [rbp + -264], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -264]
    add eax, ecx
    mov [rbp + -272], rax
    mov eax, [rbp + -272]
    mov [rbp + -32], rax
    jmp group_avg_BB1
    group_avg_BB5:
    mov rax, [rbp + -32]
    lea rdx, [rel sd_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -208], rcx
    mov eax, [rbp + -208]
    mov [rbp + -48], rax
    mov eax, 0
    mov [rbp + -216], rax
    mov eax, [rbp + -216]
    mov [rbp + -8], rax
    jmp group_avg_BB6
    group_avg_BB6:
    mov eax, [rel vv_n]
    mov [rbp + -96], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -96]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -104], rax
    mov eax, [rbp + -104]
    test eax, eax
    jne group_avg_BB7
    jmp group_avg_BB8
    group_avg_BB7:
    mov rax, [rbp + -8]
    lea rdx, [rel vv_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -112], rcx
    mov eax, [rbp + -112]
    mov ecx, [rbp + -48]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -120], rax
    mov eax, [rbp + -120]
    test eax, eax
    jne group_avg_BB10
    jmp group_avg_BB11
    group_avg_BB8:
    jmp group_avg_BB4
    group_avg_BB9:
    mov eax, 1
    mov [rbp + -192], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -192]
    add eax, ecx
    mov [rbp + -200], rax
    mov eax, [rbp + -200]
    mov [rbp + -8], rax
    jmp group_avg_BB6
    group_avg_BB10:
    mov rax, [rbp + -8]
    lea rdx, [rel vv_mark]
    mov ecx, [rdx + rax*4]
    mov [rbp + -128], rcx
    mov eax, 3
    mov [rbp + -136], rax
    mov eax, [rbp + -128]
    mov ecx, [rbp + -136]
    cmp eax, ecx
    setge al
    movzx eax, al
    mov [rbp + -144], rax
    mov eax, [rbp + -144]
    mov [rbp + -152], rax
    jmp group_avg_BB12
    group_avg_BB11:
    mov eax, 0
    mov [rbp + -152], rax
    jmp group_avg_BB12
    group_avg_BB12:
    mov eax, [rbp + -152]
    test eax, eax
    jne group_avg_BB13
    jmp group_avg_BB9
    group_avg_BB13:
    mov rax, [rbp + -8]
    lea rdx, [rel vv_mark]
    mov ecx, [rdx + rax*4]
    mov [rbp + -160], rcx
    mov eax, [rbp + -40]
    mov ecx, [rbp + -160]
    add eax, ecx
    mov [rbp + -168], rax
    mov eax, [rbp + -168]
    mov [rbp + -40], rax
    mov eax, 1
    mov [rbp + -176], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -176]
    add eax, ecx
    mov [rbp + -184], rax
    mov eax, [rbp + -184]
    mov [rbp + -16], rax
    jmp group_avg_BB9
    group_avg_BB14:
    mov eax, 10
    mov [rbp + -304], rax
    mov eax, [rbp + -40]
    mov ecx, [rbp + -304]
    imul eax, ecx
    mov [rbp + -312], rax
    mov eax, [rbp + -312]
    mov ebx, [rbp + -16]
    cdq
    idiv ebx
    mov [rbp + -320], rax
    mov eax, [rbp + -320]
    leave
    ret
    group_avg_BB15:
    mov eax, 0
    mov [rbp + -280], rax
    mov eax, [rbp + -280]
    leave
    ret
