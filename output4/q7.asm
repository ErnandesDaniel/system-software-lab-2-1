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
global q7

extern print_str
extern printf
extern putchar
extern puts
extern str_eq

    q7:
    push rbp
    mov rbp, rsp
    sub rsp, 544
    q7_BB0:
    lea rax, [q7_str_0]
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
    jmp q7_BB1
    q7_BB1:
    mov eax, 100
    mov [rbp + -56], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -56]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -64], rax
    mov eax, [rbp + -64]
    test eax, eax
    jne q7_BB2
    jmp q7_BB3
    q7_BB2:
    mov rax, [rbp + -8]
    lea rdx, [rel w]
    mov ecx, [rdx + rax*4]
    mov [rbp + -72], rcx
    mov eax, 0
    mov [rbp + -80], rax
    mov eax, [rbp + -80]
    lea rcx, [rel w]
    mov rdx, [rbp + -8]
    mov [rcx + rdx*4], eax
    mov eax, 1
    mov [rbp + -88], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -88]
    add eax, ecx
    mov [rbp + -96], rax
    mov eax, [rbp + -96]
    mov [rbp + -8], rax
    jmp q7_BB1
    q7_BB3:
    mov eax, 0
    mov [rbp + -312], rax
    mov eax, [rbp + -312]
    mov [rbp + -8], rax
    jmp q7_BB4
    q7_BB4:
    mov eax, [rel pp_n]
    mov [rbp + -104], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -104]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -112], rax
    mov eax, [rbp + -112]
    test eax, eax
    jne q7_BB5
    jmp q7_BB6
    q7_BB5:
    mov eax, 1
    mov [rbp + -280], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -280]
    add eax, ecx
    mov [rbp + -288], rax
    mov eax, [rbp + -288]
    mov [rbp + -16], rax
    jmp q7_BB7
    q7_BB6:
    mov eax, 0
    mov [rbp + -520], rax
    mov eax, [rbp + -520]
    mov [rbp + -24], rax
    mov eax, 0
    mov [rbp + -528], rax
    mov eax, [rbp + -528]
    mov [rbp + -8], rax
    jmp q7_BB14
    q7_BB7:
    mov eax, [rel pp_n]
    mov [rbp + -120], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -120]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -128], rax
    mov eax, [rbp + -128]
    test eax, eax
    jne q7_BB8
    jmp q7_BB9
    q7_BB8:
    mov rax, [rel pp_buf]
    mov [rbp + -216], rax
    mov rax, [rbp + -8]
    lea rdx, [rel pp_surname]
    mov ecx, [rdx + rax*4]
    mov [rbp + -224], rcx
    mov rax, [rbp + -16]
    lea rdx, [rel pp_surname]
    mov ecx, [rdx + rax*4]
    mov [rbp + -232], rcx
    mov rcx, [rbp + -216]
    mov edx, [rbp + -224]
    mov r8d, [rbp + -232]
    sub rsp, 32
    xor eax, eax
    call str_eq
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
    jne q7_BB11
    jmp q7_BB10
    q7_BB9:
    mov eax, 1
    mov [rbp + -296], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -296]
    add eax, ecx
    mov [rbp + -304], rax
    mov eax, [rbp + -304]
    mov [rbp + -8], rax
    jmp q7_BB4
    q7_BB10:
    mov eax, 1
    mov [rbp + -264], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -264]
    add eax, ecx
    mov [rbp + -272], rax
    mov eax, [rbp + -272]
    mov [rbp + -16], rax
    jmp q7_BB7
    q7_BB11:
    mov rax, [rel pp_buf]
    mov [rbp + -168], rax
    mov rax, [rbp + -8]
    lea rdx, [rel pp_bday]
    mov ecx, [rdx + rax*4]
    mov [rbp + -176], rcx
    mov rax, [rbp + -16]
    lea rdx, [rel pp_bday]
    mov ecx, [rdx + rax*4]
    mov [rbp + -184], rcx
    mov rcx, [rbp + -168]
    mov edx, [rbp + -176]
    mov r8d, [rbp + -184]
    sub rsp, 32
    xor eax, eax
    call str_eq
    add rsp, 32
    mov [rbp + -192], rax
    mov eax, 0
    mov [rbp + -200], rax
    mov eax, [rbp + -192]
    mov ecx, [rbp + -200]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -208], rax
    mov eax, [rbp + -208]
    test eax, eax
    jne q7_BB13
    jmp q7_BB12
    q7_BB12:
    jmp q7_BB10
    q7_BB13:
    mov rax, [rbp + -8]
    lea rdx, [rel w]
    mov ecx, [rdx + rax*4]
    mov [rbp + -136], rcx
    mov eax, 1
    mov [rbp + -144], rax
    mov eax, [rbp + -144]
    lea rcx, [rel w]
    mov rdx, [rbp + -8]
    mov [rcx + rdx*4], eax
    mov rax, [rbp + -16]
    lea rdx, [rel w]
    mov ecx, [rdx + rax*4]
    mov [rbp + -152], rcx
    mov eax, 1
    mov [rbp + -160], rax
    mov eax, [rbp + -160]
    lea rcx, [rel w]
    mov rdx, [rbp + -16]
    mov [rcx + rdx*4], eax
    jmp q7_BB12
    q7_BB14:
    mov eax, [rel pp_n]
    mov [rbp + -320], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -320]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -328], rax
    mov eax, [rbp + -328]
    test eax, eax
    jne q7_BB15
    jmp q7_BB16
    q7_BB15:
    mov rax, [rbp + -8]
    lea rdx, [rel w]
    mov ecx, [rdx + rax*4]
    mov [rbp + -480], rcx
    mov eax, 0
    mov [rbp + -488], rax
    mov eax, [rbp + -480]
    mov ecx, [rbp + -488]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -496], rax
    mov eax, [rbp + -496]
    test eax, eax
    jne q7_BB18
    jmp q7_BB17
    q7_BB16:
    lea rax, [q7_str_1]
    mov [rbp + -536], rax
    mov rcx, [rbp + -536]
    mov edx, [rbp + -24]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -544], rax
    leave
    ret
    q7_BB17:
    mov eax, 1
    mov [rbp + -504], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -504]
    add eax, ecx
    mov [rbp + -512], rax
    mov eax, [rbp + -512]
    mov [rbp + -8], rax
    jmp q7_BB14
    q7_BB18:
    mov rax, [rel pp_buf]
    mov [rbp + -336], rax
    mov rax, [rbp + -8]
    lea rdx, [rel pp_surname]
    mov ecx, [rdx + rax*4]
    mov [rbp + -344], rcx
    mov rcx, [rbp + -336]
    mov edx, [rbp + -344]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    mov eax, 44
    mov [rbp + -352], rax
    mov ecx, [rbp + -352]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -360], rax
    mov eax, 32
    mov [rbp + -368], rax
    mov ecx, [rbp + -368]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -376], rax
    mov rax, [rel pp_buf]
    mov [rbp + -384], rax
    mov rax, [rbp + -8]
    lea rdx, [rel pp_name]
    mov ecx, [rdx + rax*4]
    mov [rbp + -392], rcx
    mov rcx, [rbp + -384]
    mov edx, [rbp + -392]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    mov eax, 44
    mov [rbp + -400], rax
    mov ecx, [rbp + -400]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -408], rax
    mov eax, 32
    mov [rbp + -416], rax
    mov ecx, [rbp + -416]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -424], rax
    mov rax, [rel pp_buf]
    mov [rbp + -432], rax
    mov rax, [rbp + -8]
    lea rdx, [rel pp_bday]
    mov ecx, [rdx + rax*4]
    mov [rbp + -440], rcx
    mov rcx, [rbp + -432]
    mov edx, [rbp + -440]
    sub rsp, 32
    xor eax, eax
    call print_str
    add rsp, 32
    mov eax, 10
    mov [rbp + -448], rax
    mov ecx, [rbp + -448]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -456], rax
    mov eax, 1
    mov [rbp + -464], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -464]
    add eax, ecx
    mov [rbp + -472], rax
    mov eax, [rbp + -472]
    mov [rbp + -24], rax
    jmp q7_BB17

section .data
q7_str_0 db 61, 61, 61, 32, 81, 55, 58, 32, 83, 97, 109, 101, 32, 115, 117, 114, 110, 97, 109, 101, 44, 32, 100, 105, 102, 102, 32, 98, 100, 97, 121, 32, 61, 61, 61, 0
q7_str_1 db 71, 114, 111, 117, 112, 115, 58, 32, 37, 100, 10, 0
