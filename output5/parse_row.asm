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
global parse_row

extern col_drain
extern p_init
extern p_read
extern p_write

    parse_row:
    push rbp
    mov rbp, rsp
    sub rsp, 384
    parse_row_BB0:
    mov rax, [rel row_buf]
    mov [rbp + -40], rax
    mov eax, 0
    mov [rbp + -48], rax
    mov rcx, [rbp + -40]
    mov edx, [rbp + -48]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -56], rax
    mov eax, 0
    mov [rbp + -64], rax
    mov eax, [rbp + -56]
    mov ecx, [rbp + -64]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    test eax, eax
    jne parse_row_BB2
    jmp parse_row_BB1
    parse_row_BB1:
    lea rcx, [rel col_pipe]
    sub rsp, 32
    xor eax, eax
    call p_init
    add rsp, 32
    mov eax, 0
    mov [rbp + -232], rax
    lea rcx, [rel col_pipe]
    mov edx, [rbp + -232]
    sub rsp, 32
    xor eax, eax
    call p_write
    add rsp, 32
    mov [rbp + -240], rax
    mov eax, 0
    mov [rbp + -248], rax
    mov eax, [rbp + -248]
    mov [rbp + -8], rax
    mov eax, 0
    mov [rbp + -256], rax
    mov eax, [rbp + -256]
    mov [rbp + -24], rax
    jmp parse_row_BB3
    parse_row_BB2:
    mov eax, 0
    mov [rbp + -80], rax
    mov eax, [rbp + -80]
    leave
    ret
    parse_row_BB3:
    mov eax, 1
    mov [rbp + -88], rax
    mov eax, [rbp + -88]
    test eax, eax
    jne parse_row_BB4
    jmp parse_row_BB5
    parse_row_BB4:
    mov rax, [rel row_buf]
    mov [rbp + -96], rax
    mov rcx, [rbp + -96]
    mov edx, [rbp + -8]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -104], rax
    mov eax, [rbp + -104]
    mov [rbp + -32], rax
    mov eax, 0
    mov [rbp + -112], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -112]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -120], rax
    mov eax, [rbp + -120]
    test eax, eax
    jne parse_row_BB7
    jmp parse_row_BB6
    parse_row_BB5:
    mov eax, 1
    mov [rbp + -304], rax
    mov eax, [rbp + -304]
    neg eax
    mov [rbp + -312], rax
    lea rcx, [rel col_pipe]
    mov edx, [rbp + -312]
    sub rsp, 32
    xor eax, eax
    call p_write
    add rsp, 32
    mov [rbp + -320], rax
    mov eax, 0
    mov [rbp + -328], rax
    mov eax, [rbp + -328]
    lea rdx, [rel co]
    mov ecx, [rdx + rax*4]
    mov [rbp + -336], rcx
    lea rcx, [rel col_pipe]
    sub rsp, 32
    xor eax, eax
    call p_read
    add rsp, 32
    mov [rbp + -344], rax
    mov eax, 0
    mov [rbp + -352], rax
    mov eax, [rbp + -344]
    lea rcx, [rel co]
    mov edx, [rbp + -352]
    mov [rcx + rdx*4], eax
    mov eax, 1
    mov [rbp + -360], rax
    mov eax, [rbp + -360]
    mov [rbp + -16], rax
    jmp parse_row_BB10
    parse_row_BB6:
    mov eax, 44
    mov [rbp + -200], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -200]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -208], rax
    mov eax, [rbp + -208]
    test eax, eax
    jne parse_row_BB9
    jmp parse_row_BB8
    parse_row_BB7:
    jmp parse_row_BB5
    parse_row_BB8:
    mov eax, 1
    mov [rbp + -216], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -216]
    add eax, ecx
    mov [rbp + -224], rax
    mov eax, [rbp + -224]
    mov [rbp + -8], rax
    jmp parse_row_BB3
    parse_row_BB9:
    mov rax, [rel row_buf]
    mov [rbp + -128], rax
    mov rcx, [rbp + -128]
    mov edx, [rbp + -8]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -136], rax
    mov eax, 0
    mov [rbp + -144], rax
    mov rax, [rel row_buf]
    mov [rbp + -152], rax
    mov rcx, [rbp + -152]
    mov edx, [rbp + -8]
    mov r8d, [rbp + -144]
    mov [rcx + rdx], r8b
    mov eax, 1
    mov [rbp + -160], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -160]
    add eax, ecx
    mov [rbp + -168], rax
    lea rcx, [rel col_pipe]
    mov edx, [rbp + -168]
    sub rsp, 32
    xor eax, eax
    call p_write
    add rsp, 32
    mov [rbp + -176], rax
    mov eax, 1
    mov [rbp + -184], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -184]
    add eax, ecx
    mov [rbp + -192], rax
    mov eax, [rbp + -192]
    mov [rbp + -24], rax
    jmp parse_row_BB8
    parse_row_BB10:
    mov eax, [rbp + -16]
    mov ecx, [rbp + -24]
    cmp eax, ecx
    setle al
    movzx eax, al
    mov [rbp + -264], rax
    mov eax, [rbp + -264]
    test eax, eax
    jne parse_row_BB11
    jmp parse_row_BB12
    parse_row_BB11:
    mov eax, [rbp + -16]
    lea rdx, [rel co]
    mov ecx, [rdx + rax*4]
    mov [rbp + -272], rcx
    lea rcx, [rel col_pipe]
    sub rsp, 32
    xor eax, eax
    call p_read
    add rsp, 32
    mov [rbp + -280], rax
    mov eax, [rbp + -280]
    lea rcx, [rel co]
    mov edx, [rbp + -16]
    mov [rcx + rdx*4], eax
    mov eax, 1
    mov [rbp + -288], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -288]
    add eax, ecx
    mov [rbp + -296], rax
    mov eax, [rbp + -296]
    mov [rbp + -16], rax
    jmp parse_row_BB10
    parse_row_BB12:
    lea rcx, [rel col_pipe]
    sub rsp, 32
    xor eax, eax
    call col_drain
    add rsp, 32
    mov eax, 1
    mov [rbp + -368], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -368]
    add eax, ecx
    mov [rbp + -376], rax
    mov eax, [rbp + -376]
    leave
    ret
