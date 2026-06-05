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
global q5

extern group_avg
extern print_str
extern printf
extern putchar
extern puts
extern str_eq_lit

    q5:
    push rbp
    mov rbp, rsp
    sub rsp, 640
    q5_BB0:
    lea rax, [q5_str_0]
    mov [rbp + -80], rax
    mov rcx, [rbp + -80]
    sub rsp, 32
    xor eax, eax
    call puts
    add rsp, 32
    mov [rbp + -88], rax
    lea rax, [q5_str_1]
    mov [rbp + -96], rax
    mov rcx, [rbp + -96]
    sub rsp, 32
    xor eax, eax
    call group_avg
    add rsp, 32
    mov [rbp + -104], rax
    mov eax, [rbp + -104]
    mov [rbp + -56], rax
    lea rax, [q5_str_2]
    mov [rbp + -112], rax
    mov rcx, [rbp + -112]
    mov edx, [rbp + -56]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -120], rax
    mov eax, 0
    mov [rbp + -128], rax
    mov eax, [rbp + -128]
    mov [rbp + -8], rax
    mov eax, 0
    mov [rbp + -136], rax
    mov eax, [rbp + -136]
    mov [rbp + -24], rax
    jmp q5_BB1
    q5_BB1:
    mov eax, [rel sd_n]
    mov [rbp + -144], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -144]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -152], rax
    mov eax, [rbp + -152]
    test eax, eax
    jne q5_BB2
    jmp q5_BB3
    q5_BB2:
    mov rax, [rel sd_buf]
    mov [rbp + -568], rax
    mov eax, [rbp + -24]
    lea rdx, [rel sd_group]
    mov ecx, [rdx + rax*4]
    mov [rbp + -576], rcx
    lea rax, [q5_str_3]
    mov [rbp + -584], rax
    mov rcx, [rbp + -568]
    mov edx, [rbp + -576]
    mov r8, [rbp + -584]
    sub rsp, 32
    xor eax, eax
    call str_eq_lit
    add rsp, 32
    mov [rbp + -592], rax
    mov eax, 0
    mov [rbp + -600], rax
    mov eax, [rbp + -592]
    mov ecx, [rbp + -600]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -608], rax
    mov eax, [rbp + -608]
    test eax, eax
    jne q5_BB5
    jmp q5_BB4
    q5_BB3:
    lea rax, [q5_str_4]
    mov [rbp + -632], rax
    mov rcx, [rbp + -632]
    mov edx, [rbp + -8]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -640], rax
    leave
    ret
    q5_BB4:
    mov eax, 1
    mov [rbp + -616], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -616]
    add eax, ecx
    mov [rbp + -624], rax
    mov eax, [rbp + -624]
    mov [rbp + -24], rax
    jmp q5_BB1
    q5_BB5:
    mov eax, [rbp + -24]
    lea rdx, [rel sd_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -272], rcx
    mov eax, [rbp + -272]
    mov [rbp + -64], rax
    mov eax, 0
    mov [rbp + -280], rax
    mov eax, [rbp + -280]
    mov [rbp + -16], rax
    mov eax, 0
    mov [rbp + -288], rax
    mov eax, [rbp + -288]
    mov [rbp + -72], rax
    mov eax, 0
    mov [rbp + -296], rax
    mov eax, [rbp + -296]
    mov [rbp + -32], rax
    jmp q5_BB6
    q5_BB6:
    mov eax, [rel vv_n]
    mov [rbp + -160], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -160]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -168], rax
    mov eax, [rbp + -168]
    test eax, eax
    jne q5_BB7
    jmp q5_BB8
    q5_BB7:
    mov eax, [rbp + -32]
    lea rdx, [rel vv_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -176], rcx
    mov eax, [rbp + -176]
    mov ecx, [rbp + -64]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -184], rax
    mov eax, [rbp + -184]
    test eax, eax
    jne q5_BB10
    jmp q5_BB11
    q5_BB8:
    mov eax, 0
    mov [rbp + -552], rax
    mov eax, [rbp + -72]
    mov ecx, [rbp + -552]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -560], rax
    mov eax, [rbp + -560]
    test eax, eax
    jne q5_BB15
    jmp q5_BB14
    q5_BB9:
    mov eax, 1
    mov [rbp + -256], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -256]
    add eax, ecx
    mov [rbp + -264], rax
    mov eax, [rbp + -264]
    mov [rbp + -32], rax
    jmp q5_BB6
    q5_BB10:
    mov eax, [rbp + -32]
    lea rdx, [rel vv_mark]
    mov ecx, [rdx + rax*4]
    mov [rbp + -192], rcx
    mov eax, 3
    mov [rbp + -200], rax
    mov eax, [rbp + -192]
    mov ecx, [rbp + -200]
    cmp eax, ecx
    setge al
    movzx eax, al
    mov [rbp + -208], rax
    mov eax, [rbp + -208]
    mov [rbp + -216], rax
    jmp q5_BB12
    q5_BB11:
    mov eax, 0
    mov [rbp + -216], rax
    jmp q5_BB12
    q5_BB12:
    mov eax, [rbp + -216]
    test eax, eax
    jne q5_BB13
    jmp q5_BB9
    q5_BB13:
    mov eax, [rbp + -32]
    lea rdx, [rel vv_mark]
    mov ecx, [rdx + rax*4]
    mov [rbp + -224], rcx
    mov eax, [rbp + -16]
    mov ecx, [rbp + -224]
    add eax, ecx
    mov [rbp + -232], rax
    mov eax, [rbp + -232]
    mov [rbp + -16], rax
    mov eax, 1
    mov [rbp + -240], rax
    mov eax, [rbp + -72]
    mov ecx, [rbp + -240]
    add eax, ecx
    mov [rbp + -248], rax
    mov eax, [rbp + -248]
    mov [rbp + -72], rax
    jmp q5_BB9
    q5_BB14:
    jmp q5_BB4
    q5_BB15:
    mov eax, 10
    mov [rbp + -520], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -520]
    imul eax, ecx
    mov [rbp + -528], rax
    mov eax, [rbp + -528]
    mov ebx, [rbp + -72]
    cdq
    idiv ebx
    mov [rbp + -536], rax
    mov eax, [rbp + -536]
    mov [rbp + -48], rax
    mov eax, [rbp + -48]
    mov ecx, [rbp + -56]
    cmp eax, ecx
    setge al
    movzx eax, al
    mov [rbp + -544], rax
    mov eax, [rbp + -544]
    test eax, eax
    jne q5_BB17
    jmp q5_BB16
    q5_BB16:
    jmp q5_BB14
    q5_BB17:
    mov eax, 0
    mov [rbp + -512], rax
    mov eax, [rbp + -512]
    mov [rbp + -40], rax
    jmp q5_BB18
    q5_BB18:
    mov eax, [rel pp_n]
    mov [rbp + -304], rax
    mov eax, [rbp + -40]
    mov ecx, [rbp + -304]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -312], rax
    mov eax, [rbp + -312]
    test eax, eax
    jne q5_BB19
    jmp q5_BB20
    q5_BB19:
    mov eax, [rbp + -40]
    lea rdx, [rel pp_id]
    mov ecx, [rdx + rax*4]
    mov [rbp + -480], rcx
    mov eax, [rbp + -480]
    mov ecx, [rbp + -64]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -488], rax
    mov eax, [rbp + -488]
    test eax, eax
    jne q5_BB22
    jmp q5_BB21
    q5_BB20:
    jmp q5_BB16
    q5_BB21:
    mov eax, 1
    mov [rbp + -496], rax
    mov eax, [rbp + -40]
    mov ecx, [rbp + -496]
    add eax, ecx
    mov [rbp + -504], rax
    mov eax, [rbp + -504]
    mov [rbp + -40], rax
    jmp q5_BB18
    q5_BB22:
    lea rax, [q5_str_5]
    mov [rbp + -320], rax
    mov rcx, [rbp + -320]
    mov edx, [rbp + -64]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -328], rax
    mov eax, 44
    mov [rbp + -336], rax
    mov ecx, [rbp + -336]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -344], rax
    mov eax, 32
    mov [rbp + -352], rax
    mov ecx, [rbp + -352]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -360], rax
    mov rax, [rel pp_buf]
    mov [rbp + -368], rax
    mov eax, [rbp + -40]
    lea rdx, [rel pp_surname]
    mov ecx, [rdx + rax*4]
    mov [rbp + -376], rcx
    mov rcx, [rbp + -368]
    mov edx, [rbp + -376]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    mov eax, 32
    mov [rbp + -384], rax
    mov ecx, [rbp + -384]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -392], rax
    mov rax, [rel pp_buf]
    mov [rbp + -400], rax
    mov eax, [rbp + -40]
    lea rdx, [rel pp_name]
    mov ecx, [rdx + rax*4]
    mov [rbp + -408], rcx
    mov rcx, [rbp + -400]
    mov edx, [rbp + -408]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    lea rax, [q5_str_6]
    mov [rbp + -416], rax
    mov eax, 10
    mov [rbp + -424], rax
    mov eax, [rbp + -48]
    mov ebx, [rbp + -424]
    cdq
    idiv ebx
    mov [rbp + -432], rax
    mov eax, 10
    mov [rbp + -440], rax
    mov eax, [rbp + -48]
    mov ebx, [rbp + -440]
    cdq
    idiv ebx
    mov [rbp + -448], rdx
    mov rcx, [rbp + -416]
    mov edx, [rbp + -432]
    mov r8d, [rbp + -448]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -456], rax
    mov eax, 1
    mov [rbp + -464], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -464]
    add eax, ecx
    mov [rbp + -472], rax
    mov eax, [rbp + -472]
    mov [rbp + -8], rax
    jmp q5_BB21

section .data
q5_str_0 db 61, 61, 61, 32, 81, 53, 58, 32, 65, 118, 103, 32, 103, 114, 97, 100, 101, 115, 32, 52, 49, 48, 48, 32, 62, 61, 32, 49, 49, 48, 48, 32, 61, 61, 61, 0
q5_str_1 db 49, 49, 48, 48, 0
q5_str_2 db 65, 118, 103, 32, 49, 49, 48, 48, 32, 61, 32, 37, 100, 10, 0
q5_str_3 db 52, 49, 48, 48, 0
q5_str_4 db 70, 111, 117, 110, 100, 58, 32, 37, 100, 10, 0
q5_str_5 db 37, 100, 0
q5_str_6 db 32, 37, 100, 46, 37, 100, 10, 0
