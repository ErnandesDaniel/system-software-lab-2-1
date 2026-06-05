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
global q6

extern print_str
extern printf
extern putchar
extern puts
extern str_eq_lit
extern str_gt_lit

    q6:
    push rbp
    mov rbp, rsp
    sub rsp, 560
    q6_BB0:
    lea rax, [q6_str_0]
    mov [rbp + -48], rax
    mov rcx, [rbp + -48]
    sub rsp, 32
    xor eax, eax
    call puts
    add rsp, 32
    mov [rbp + -56], rax
    mov eax, 0
    mov [rbp + -64], rax
    mov eax, [rbp + -64]
    mov [rbp + -32], rax
    mov eax, 0
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    mov [rbp + -40], rax
    jmp q6_BB1
    q6_BB1:
    mov eax, [rel sd_n]
    mov [rbp + -80], rax
    mov eax, [rbp + -40]
    mov ecx, [rbp + -80]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -88], rax
    mov eax, [rbp + -88]
    test eax, eax
    jne q6_BB2
    jmp q6_BB3
    q6_BB2:
    mov rax, [rel sd_buf]
    mov [rbp + -480], rax
    mov rax, [rbp + -40]
    lea rdx, [rel sd_start]
    mov ecx, [rdx + rax*4]
    mov [rbp + -488], rcx
    lea rax, [q6_str_1]
    mov [rbp + -496], rax
    mov rcx, [rbp + -480]
    mov edx, [rbp + -488]
    mov r8, [rbp + -496]
    sub rsp, 32
    xor eax, eax
    call str_gt_lit
    add rsp, 32
    mov [rbp + -504], rax
    mov eax, 0
    mov [rbp + -512], rax
    mov eax, [rbp + -504]
    mov ecx, [rbp + -512]
    cmp eax, ecx
    setg al
    movzx eax, al
    mov [rbp + -520], rax
    mov eax, [rbp + -520]
    test eax, eax
    jne q6_BB5
    jmp q6_BB4
    q6_BB3:
    lea rax, [q6_str_2]
    mov [rbp + -544], rax
    mov rcx, [rbp + -544]
    mov edx, [rbp + -32]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -552], rax
    leave
    ret
    q6_BB4:
    mov eax, 1
    mov [rbp + -528], rax
    mov eax, [rbp + -40]
    mov ecx, [rbp + -528]
    add eax, ecx
    mov [rbp + -536], rax
    mov eax, [rbp + -536]
    mov [rbp + -40], rax
    jmp q6_BB1
    q6_BB5:
    mov rax, [rbp + -40]
    lea rdx, [rel sd_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -464], rcx
    mov eax, [rbp + -464]
    mov [rbp + -16], rax
    mov eax, 0
    mov [rbp + -472], rax
    mov eax, [rbp + -472]
    mov [rbp + -8], rax
    jmp q6_BB6
    q6_BB6:
    mov eax, [rel st_n]
    mov [rbp + -96], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -96]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -104], rax
    mov eax, [rbp + -104]
    test eax, eax
    jne q6_BB7
    jmp q6_BB8
    q6_BB7:
    mov rax, [rbp + -8]
    lea rdx, [rel st_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -112], rcx
    mov eax, [rbp + -112]
    mov ecx, [rbp + -16]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -120], rax
    mov eax, [rbp + -120]
    test eax, eax
    jne q6_BB13
    jmp q6_BB14
    q6_BB8:
    jmp q6_BB4
    q6_BB9:
    mov eax, 1
    mov [rbp + -448], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -448]
    add eax, ecx
    mov [rbp + -456], rax
    mov eax, [rbp + -456]
    mov [rbp + -8], rax
    jmp q6_BB6
    q6_BB10:
    mov rax, [rbp + -8]
    lea rdx, [rel st_course]
    mov ecx, [rdx + rax*4]
    mov [rbp + -184], rcx
    mov eax, 1
    mov [rbp + -192], rax
    mov eax, [rbp + -184]
    mov ecx, [rbp + -192]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -200], rax
    mov eax, [rbp + -200]
    mov [rbp + -208], rax
    jmp q6_BB12
    q6_BB11:
    mov eax, 0
    mov [rbp + -208], rax
    jmp q6_BB12
    q6_BB12:
    mov eax, [rbp + -208]
    test eax, eax
    jne q6_BB16
    jmp q6_BB9
    q6_BB13:
    mov rax, [rel st_buf]
    mov [rbp + -128], rax
    mov rax, [rbp + -8]
    lea rdx, [rel st_form]
    mov ecx, [rdx + rax*4]
    mov [rbp + -136], rcx
    lea rax, [q6_str_3]
    mov [rbp + -144], rax
    mov rcx, [rbp + -128]
    mov edx, [rbp + -136]
    mov r8, [rbp + -144]
    sub rsp, 32
    xor eax, eax
    call str_eq_lit
    add rsp, 32
    mov [rbp + -152], rax
    mov eax, 0
    mov [rbp + -160], rax
    mov eax, [rbp + -152]
    mov ecx, [rbp + -160]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -168], rax
    mov eax, [rbp + -168]
    mov [rbp + -176], rax
    jmp q6_BB15
    q6_BB14:
    mov eax, 0
    mov [rbp + -176], rax
    jmp q6_BB15
    q6_BB15:
    mov eax, [rbp + -176]
    test eax, eax
    jne q6_BB10
    jmp q6_BB11
    q6_BB16:
    mov eax, 1
    mov [rbp + -424], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -424]
    add eax, ecx
    mov [rbp + -432], rax
    mov eax, [rbp + -432]
    mov [rbp + -32], rax
    mov eax, 0
    mov [rbp + -440], rax
    mov eax, [rbp + -440]
    mov [rbp + -24], rax
    jmp q6_BB17
    q6_BB17:
    mov eax, [rel pp_n]
    mov [rbp + -216], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -216]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -224], rax
    mov eax, [rbp + -224]
    test eax, eax
    jne q6_BB18
    jmp q6_BB19
    q6_BB18:
    mov rax, [rbp + -24]
    lea rdx, [rel pp_id]
    mov ecx, [rdx + rax*4]
    mov [rbp + -392], rcx
    mov eax, [rbp + -392]
    mov ecx, [rbp + -16]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -400], rax
    mov eax, [rbp + -400]
    test eax, eax
    jne q6_BB21
    jmp q6_BB20
    q6_BB19:
    jmp q6_BB8
    q6_BB20:
    mov eax, 1
    mov [rbp + -408], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -408]
    add eax, ecx
    mov [rbp + -416], rax
    mov eax, [rbp + -416]
    mov [rbp + -24], rax
    jmp q6_BB17
    q6_BB21:
    mov rax, [rel sd_buf]
    mov [rbp + -232], rax
    mov rax, [rbp + -40]
    lea rdx, [rel sd_group]
    mov ecx, [rdx + rax*4]
    mov [rbp + -240], rcx
    mov rcx, [rbp + -232]
    mov edx, [rbp + -240]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    mov eax, 44
    mov [rbp + -248], rax
    mov ecx, [rbp + -248]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -256], rax
    mov eax, 32
    mov [rbp + -264], rax
    mov ecx, [rbp + -264]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -272], rax
    lea rax, [q6_str_4]
    mov [rbp + -280], rax
    mov rcx, [rbp + -280]
    mov edx, [rbp + -16]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -288], rax
    mov eax, 44
    mov [rbp + -296], rax
    mov ecx, [rbp + -296]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -304], rax
    mov eax, 32
    mov [rbp + -312], rax
    mov ecx, [rbp + -312]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -320], rax
    mov rax, [rel pp_buf]
    mov [rbp + -328], rax
    mov rax, [rbp + -24]
    lea rdx, [rel pp_surname]
    mov ecx, [rdx + rax*4]
    mov [rbp + -336], rcx
    mov rcx, [rbp + -328]
    mov edx, [rbp + -336]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    mov eax, 32
    mov [rbp + -344], rax
    mov ecx, [rbp + -344]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -352], rax
    mov rax, [rel pp_buf]
    mov [rbp + -360], rax
    mov rax, [rbp + -24]
    lea rdx, [rel pp_name]
    mov ecx, [rdx + rax*4]
    mov [rbp + -368], rcx
    mov rcx, [rbp + -360]
    mov edx, [rbp + -368]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    mov eax, 10
    mov [rbp + -376], rax
    mov ecx, [rbp + -376]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -384], rax
    jmp q6_BB20

section .data
q6_str_0 db 61, 61, 61, 32, 81, 54, 58, 32, 69, 110, 114, 111, 108, 108, 101, 100, 32, 97, 102, 116, 101, 114, 32, 50, 48, 49, 50, 45, 48, 57, 45, 48, 49, 44, 32, 49, 32, 99, 111, 117, 114, 115, 101, 44, 32, 112, 97, 114, 116, 45, 116, 105, 109, 101, 32, 61, 61, 61, 0
q6_str_1 db 50, 48, 49, 50, 45, 48, 57, 45, 48, 49, 0
q6_str_2 db 67, 111, 117, 110, 116, 58, 32, 37, 100, 10, 0
q6_str_3 db 112, 97, 114, 116, 45, 116, 105, 109, 101, 0
q6_str_4 db 37, 100, 0
