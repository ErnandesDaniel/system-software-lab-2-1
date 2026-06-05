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
global q3

extern printf
extern puts
extern str_empty
extern str_eq_lit

    q3:
    push rbp
    mov rbp, rsp
    sub rsp, 352
    q3_BB0:
    lea rax, [q3_str_0]
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
    mov eax, 0
    mov [rbp + -64], rax
    mov eax, [rbp + -64]
    mov [rbp + -24], rax
    jmp q3_BB1
    q3_BB1:
    mov eax, [rel pp_n]
    mov [rbp + -72], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -72]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -80], rax
    mov eax, [rbp + -80]
    test eax, eax
    jne q3_BB2
    jmp q3_BB3
    q3_BB2:
    mov rax, [rel pp_buf]
    mov [rbp + -288], rax
    mov eax, [rbp + -24]
    lea rdx, [rel pp_patr]
    mov ecx, [rdx + rax*4]
    mov [rbp + -296], rcx
    mov rcx, [rbp + -288]
    mov edx, [rbp + -296]
    sub rsp, 32
    xor eax, eax
    call str_empty
    add rsp, 32
    mov [rbp + -304], rax
    mov eax, 0
    mov [rbp + -312], rax
    mov eax, [rbp + -304]
    mov ecx, [rbp + -312]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -320], rax
    mov eax, [rbp + -320]
    test eax, eax
    jne q3_BB5
    jmp q3_BB4
    q3_BB3:
    lea rax, [q3_str_1]
    mov [rbp + -344], rax
    mov rcx, [rbp + -344]
    mov edx, [rbp + -8]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -352], rax
    leave
    ret
    q3_BB4:
    mov eax, 1
    mov [rbp + -328], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -328]
    add eax, ecx
    mov [rbp + -336], rax
    mov eax, [rbp + -336]
    mov [rbp + -24], rax
    jmp q3_BB1
    q3_BB5:
    mov eax, 0
    mov [rbp + -280], rax
    mov eax, [rbp + -280]
    mov [rbp + -16], rax
    jmp q3_BB6
    q3_BB6:
    mov eax, [rel st_n]
    mov [rbp + -88], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -88]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -96], rax
    mov eax, [rbp + -96]
    test eax, eax
    jne q3_BB7
    jmp q3_BB8
    q3_BB7:
    mov eax, [rbp + -16]
    lea rdx, [rel st_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -104], rcx
    mov eax, [rbp + -24]
    lea rdx, [rel pp_id]
    mov ecx, [rdx + rax*4]
    mov [rbp + -112], rcx
    mov eax, [rbp + -104]
    mov ecx, [rbp + -112]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -120], rax
    mov eax, [rbp + -120]
    test eax, eax
    jne q3_BB10
    jmp q3_BB11
    q3_BB8:
    jmp q3_BB4
    q3_BB9:
    mov eax, 1
    mov [rbp + -264], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -264]
    add eax, ecx
    mov [rbp + -272], rax
    mov eax, [rbp + -272]
    mov [rbp + -16], rax
    jmp q3_BB6
    q3_BB10:
    mov rax, [rel st_buf]
    mov [rbp + -128], rax
    mov eax, [rbp + -16]
    lea rdx, [rel st_fac]
    mov ecx, [rdx + rax*4]
    mov [rbp + -136], rcx
    lea rax, [q3_str_2]
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
    jmp q3_BB12
    q3_BB11:
    mov eax, 0
    mov [rbp + -176], rax
    jmp q3_BB12
    q3_BB12:
    mov eax, [rbp + -176]
    test eax, eax
    jne q3_BB13
    jmp q3_BB9
    q3_BB13:
    mov eax, 0
    mov [rbp + -256], rax
    mov eax, [rbp + -256]
    mov [rbp + -32], rax
    jmp q3_BB14
    q3_BB14:
    mov eax, [rel sd_n]
    mov [rbp + -184], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -184]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -192], rax
    mov eax, [rbp + -192]
    test eax, eax
    jne q3_BB15
    jmp q3_BB16
    q3_BB15:
    mov eax, [rbp + -32]
    lea rdx, [rel sd_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -216], rcx
    mov eax, [rbp + -24]
    lea rdx, [rel pp_id]
    mov ecx, [rdx + rax*4]
    mov [rbp + -224], rcx
    mov eax, [rbp + -216]
    mov ecx, [rbp + -224]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -232], rax
    mov eax, [rbp + -232]
    test eax, eax
    jne q3_BB18
    jmp q3_BB17
    q3_BB16:
    jmp q3_BB8
    q3_BB17:
    mov eax, 1
    mov [rbp + -240], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -240]
    add eax, ecx
    mov [rbp + -248], rax
    mov eax, [rbp + -248]
    mov [rbp + -32], rax
    jmp q3_BB14
    q3_BB18:
    mov eax, 1
    mov [rbp + -200], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -200]
    add eax, ecx
    mov [rbp + -208], rax
    mov eax, [rbp + -208]
    mov [rbp + -8], rax
    jmp q3_BB16

section .data
q3_str_0 db 61, 61, 61, 32, 81, 51, 58, 32, 67, 111, 117, 110, 116, 32, 70, 67, 69, 32, 119, 105, 116, 104, 111, 117, 116, 32, 112, 97, 116, 114, 111, 110, 121, 109, 105, 99, 32, 61, 61, 61, 0
q3_str_1 db 37, 100, 10, 0
q3_str_2 db 70, 67, 69, 0
