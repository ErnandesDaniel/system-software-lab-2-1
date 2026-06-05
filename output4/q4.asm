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
global q4

extern printf
extern puts
extern str_eq_lit

    q4:
    push rbp
    mov rbp, rsp
    sub rsp, 416
    q4_BB0:
    lea rax, [q4_str_0]
    mov [rbp + -40], rax
    mov rcx, [rbp + -40]
    sub rsp, 32
    xor eax, eax
    call puts
    add rsp, 32
    mov [rbp + -48], rax
    mov eax, 0
    mov [rbp + -56], rax
    mov eax, [rbp + -56]
    mov [rbp + -8], rax
    jmp q4_BB1
    q4_BB1:
    mov eax, 50
    mov [rbp + -64], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -64]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    test eax, eax
    jne q4_BB2
    jmp q4_BB3
    q4_BB2:
    mov rax, [rbp + -8]
    lea rdx, [rel w]
    mov ecx, [rdx + rax*4]
    mov [rbp + -80], rcx
    mov eax, 0
    mov [rbp + -88], rax
    mov eax, [rbp + -88]
    lea rcx, [rel w]
    mov rdx, [rbp + -8]
    mov [rcx + rdx*4], eax
    mov eax, 1
    mov [rbp + -96], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -96]
    add eax, ecx
    mov [rbp + -104], rax
    mov eax, [rbp + -104]
    mov [rbp + -8], rax
    jmp q4_BB1
    q4_BB3:
    mov eax, 0
    mov [rbp + -416], rax
    mov eax, [rbp + -416]
    mov [rbp + -8], rax
    jmp q4_BB4
    q4_BB4:
    mov eax, [rel gp_n]
    mov [rbp + -112], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -112]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -120], rax
    mov eax, [rbp + -120]
    test eax, eax
    jne q4_BB5
    jmp q4_BB6
    q4_BB5:
    mov rax, [rbp + -8]
    lea rdx, [rel w]
    mov ecx, [rdx + rax*4]
    mov [rbp + -128], rcx
    mov eax, 0
    mov [rbp + -136], rax
    mov eax, [rbp + -128]
    mov ecx, [rbp + -136]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -144], rax
    mov eax, [rbp + -144]
    test eax, eax
    jne q4_BB8
    jmp q4_BB9
    q4_BB6:
    leave
    ret
    q4_BB7:
    mov eax, 1
    mov [rbp + -400], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -400]
    add eax, ecx
    mov [rbp + -408], rax
    mov eax, [rbp + -408]
    mov [rbp + -8], rax
    jmp q4_BB4
    q4_BB8:
    mov rax, [rel gp_buf]
    mov [rbp + -152], rax
    mov rax, [rbp + -8]
    lea rdx, [rel gp_dept]
    mov ecx, [rdx + rax*4]
    mov [rbp + -160], rcx
    lea rax, [q4_str_1]
    mov [rbp + -168], rax
    mov rcx, [rbp + -152]
    mov edx, [rbp + -160]
    mov r8, [rbp + -168]
    sub rsp, 32
    xor eax, eax
    call str_eq_lit
    add rsp, 32
    mov [rbp + -176], rax
    mov eax, 0
    mov [rbp + -184], rax
    mov eax, [rbp + -176]
    mov ecx, [rbp + -184]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -192], rax
    mov eax, [rbp + -192]
    mov [rbp + -200], rax
    jmp q4_BB10
    q4_BB9:
    mov eax, 0
    mov [rbp + -200], rax
    jmp q4_BB10
    q4_BB10:
    mov eax, [rbp + -200]
    test eax, eax
    jne q4_BB11
    jmp q4_BB7
    q4_BB11:
    mov rax, [rbp + -8]
    lea rdx, [rel gp_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -344], rcx
    mov eax, [rbp + -344]
    mov [rbp + -24], rax
    mov eax, 0
    mov [rbp + -352], rax
    mov eax, [rbp + -352]
    mov [rbp + -16], rax
    mov eax, 0
    mov [rbp + -360], rax
    mov eax, [rbp + -360]
    mov [rbp + -32], rax
    jmp q4_BB12
    q4_BB12:
    mov eax, [rel gp_n]
    mov [rbp + -208], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -208]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -216], rax
    mov eax, [rbp + -216]
    test eax, eax
    jne q4_BB13
    jmp q4_BB14
    q4_BB13:
    mov rax, [rbp + -32]
    lea rdx, [rel gp_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -224], rcx
    mov eax, [rbp + -224]
    mov ecx, [rbp + -24]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -232], rax
    mov eax, [rbp + -232]
    test eax, eax
    jne q4_BB16
    jmp q4_BB17
    q4_BB14:
    mov eax, 2
    mov [rbp + -384], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -384]
    cmp eax, ecx
    setg al
    movzx eax, al
    mov [rbp + -392], rax
    mov eax, [rbp + -392]
    test eax, eax
    jne q4_BB21
    jmp q4_BB20
    q4_BB15:
    mov eax, 1
    mov [rbp + -328], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -328]
    add eax, ecx
    mov [rbp + -336], rax
    mov eax, [rbp + -336]
    mov [rbp + -32], rax
    jmp q4_BB12
    q4_BB16:
    mov rax, [rel gp_buf]
    mov [rbp + -240], rax
    mov rax, [rbp + -32]
    lea rdx, [rel gp_dept]
    mov ecx, [rdx + rax*4]
    mov [rbp + -248], rcx
    lea rax, [q4_str_2]
    mov [rbp + -256], rax
    mov rcx, [rbp + -240]
    mov edx, [rbp + -248]
    mov r8, [rbp + -256]
    sub rsp, 32
    xor eax, eax
    call str_eq_lit
    add rsp, 32
    mov [rbp + -264], rax
    mov eax, 0
    mov [rbp + -272], rax
    mov eax, [rbp + -264]
    mov ecx, [rbp + -272]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -280], rax
    mov eax, [rbp + -280]
    mov [rbp + -288], rax
    jmp q4_BB18
    q4_BB17:
    mov eax, 0
    mov [rbp + -288], rax
    jmp q4_BB18
    q4_BB18:
    mov eax, [rbp + -288]
    test eax, eax
    jne q4_BB19
    jmp q4_BB15
    q4_BB19:
    mov eax, 1
    mov [rbp + -296], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -296]
    add eax, ecx
    mov [rbp + -304], rax
    mov eax, [rbp + -304]
    mov [rbp + -16], rax
    mov rax, [rbp + -32]
    lea rdx, [rel w]
    mov ecx, [rdx + rax*4]
    mov [rbp + -312], rcx
    mov eax, 1
    mov [rbp + -320], rax
    mov eax, [rbp + -320]
    lea rcx, [rel w]
    mov rdx, [rbp + -32]
    mov [rcx + rdx*4], eax
    jmp q4_BB15
    q4_BB20:
    jmp q4_BB7
    q4_BB21:
    lea rax, [q4_str_3]
    mov [rbp + -368], rax
    mov rcx, [rbp + -368]
    mov edx, [rbp + -24]
    mov r8d, [rbp + -16]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -376], rax
    jmp q4_BB20

section .data
q4_str_0 db 61, 61, 61, 32, 81, 52, 58, 32, 80, 108, 97, 110, 115, 32, 62, 50, 32, 103, 114, 111, 117, 112, 115, 32, 111, 110, 32, 67, 69, 32, 61, 61, 61, 0
q4_str_1 db 68, 101, 112, 97, 114, 116, 109, 101, 110, 116, 32, 111, 102, 32, 67, 111, 109, 112, 117, 116, 101, 114, 32, 69, 110, 103, 105, 110, 101, 101, 114, 105, 110, 103, 0
q4_str_2 db 68, 101, 112, 97, 114, 116, 109, 101, 110, 116, 32, 111, 102, 32, 67, 111, 109, 112, 117, 116, 101, 114, 32, 69, 110, 103, 105, 110, 101, 101, 114, 105, 110, 103, 0
q4_str_3 db 37, 100, 58, 32, 37, 100, 32, 103, 114, 111, 117, 112, 115, 10, 0
