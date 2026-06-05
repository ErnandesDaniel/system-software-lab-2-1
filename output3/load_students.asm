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
global load_students

extern atoi
extern fclose
extern fopen
extern malloc
extern parse_row
extern read_line
extern str_save

    load_students:
    push rbp
    mov rbp, rsp
    sub rsp, 400
    mov [rbp + -8], rcx
    load_students_BB0:
    lea rax, [load_students_str_0]
    mov [rbp + -32], rax
    mov rcx, [rbp + -8]
    mov rdx, [rbp + -32]
    sub rsp, 32
    xor eax, eax
    call fopen
    add rsp, 32
    mov [rbp + -40], rax
    mov rax, [rbp + -40]
    mov [rbp + -16], rax
    mov eax, 0
    mov [rbp + -48], rax
    mov rax, [rbp + -16]
    mov rcx, [rbp + -48]
    cmp rax, rcx
    sete al
    movzx eax, al
    mov [rbp + -56], rax
    mov eax, [rbp + -56]
    test eax, eax
    jne load_students_BB2
    jmp load_students_BB1
    load_students_BB1:
    mov rcx, [rbp + -16]
    sub rsp, 32
    xor eax, eax
    call read_line
    add rsp, 32
    mov [rbp + -336], rax
    mov eax, [rel sd_n]
    mov [rbp + -344], rax
    mov eax, 0
    mov [rbp + -352], rax
    mov eax, [rbp + -352]
    mov [rel sd_n], eax
    mov rax, [rel sd_buf]
    mov [rbp + -360], rax
    mov eax, 5000
    mov [rbp + -368], rax
    mov ecx, [rbp + -368]
    sub rsp, 32
    xor eax, eax
    call malloc
    add rsp, 32
    mov [rbp + -376], rax
    mov rax, [rbp + -376]
    mov [rel sd_buf], rax
    mov eax, 0
    mov [rbp + -384], rax
    mov eax, [rbp + -384]
    mov [rbp + -24], rax
    jmp load_students_BB3
    load_students_BB2:
    mov eax, 1
    mov [rbp + -64], rax
    mov eax, [rbp + -64]
    neg eax
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    leave
    ret
    load_students_BB3:
    mov eax, 1
    mov [rbp + -80], rax
    mov eax, [rbp + -80]
    test eax, eax
    jne load_students_BB4
    jmp load_students_BB5
    load_students_BB4:
    mov rcx, [rbp + -16]
    sub rsp, 32
    xor eax, eax
    call read_line
    add rsp, 32
    mov [rbp + -88], rax
    mov eax, 1
    mov [rbp + -96], rax
    mov eax, [rbp + -96]
    neg eax
    mov [rbp + -104], rax
    mov eax, [rbp + -88]
    mov ecx, [rbp + -104]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -112], rax
    mov eax, [rbp + -112]
    test eax, eax
    jne load_students_BB7
    jmp load_students_BB6
    load_students_BB5:
    mov rcx, [rbp + -16]
    sub rsp, 32
    xor eax, eax
    call fclose
    add rsp, 32
    mov [rbp + -392], rax
    mov eax, [rel sd_n]
    mov [rbp + -400], rax
    mov eax, [rbp + -400]
    leave
    ret
    load_students_BB6:
    sub rsp, 32
    xor eax, eax
    call parse_row
    add rsp, 32
    mov [rbp + -120], rax
    mov eax, [rel sd_n]
    mov [rbp + -128], rax
    mov rax, [rbp + -128]
    lea rdx, [rel sd_pid]
    mov ecx, [rdx + rax*4]
    mov [rbp + -136], rcx
    mov rax, [rel row_buf]
    mov [rbp + -144], rax
    mov eax, 0
    mov [rbp + -152], rax
    mov rax, [rbp + -152]
    lea rdx, [rel co]
    mov ecx, [rdx + rax*4]
    mov [rbp + -160], rcx
    mov rax, [rbp + -144]
    mov rcx, [rbp + -160]
    add rax, rcx
    mov [rbp + -168], rax
    mov rcx, [rbp + -168]
    sub rsp, 32
    xor eax, eax
    call atoi
    add rsp, 32
    mov [rbp + -176], rax
    mov eax, [rel sd_n]
    mov [rbp + -184], rax
    mov eax, [rbp + -176]
    lea rcx, [rel sd_pid]
    mov rdx, [rbp + -184]
    mov [rcx + rdx*4], eax
    mov eax, [rel sd_n]
    mov [rbp + -192], rax
    mov rax, [rbp + -192]
    lea rdx, [rel sd_group]
    mov ecx, [rdx + rax*4]
    mov [rbp + -200], rcx
    mov eax, [rel sd_n]
    mov [rbp + -208], rax
    mov eax, [rbp + -24]
    lea rcx, [rel sd_group]
    mov rdx, [rbp + -208]
    mov [rcx + rdx*4], eax
    mov rax, [rel sd_buf]
    mov [rbp + -216], rax
    mov eax, 1
    mov [rbp + -224], rax
    mov rax, [rbp + -224]
    lea rdx, [rel co]
    mov ecx, [rdx + rax*4]
    mov [rbp + -232], rcx
    mov rcx, [rbp + -216]
    mov edx, [rbp + -24]
    mov r8d, [rbp + -232]
    sub rsp, 32
    xor eax, eax
    call str_save
    add rsp, 32
    mov [rbp + -240], rax
    mov eax, [rbp + -240]
    mov [rbp + -24], rax
    mov eax, [rel sd_n]
    mov [rbp + -248], rax
    mov rax, [rbp + -248]
    lea rdx, [rel sd_start]
    mov ecx, [rdx + rax*4]
    mov [rbp + -256], rcx
    mov eax, [rel sd_n]
    mov [rbp + -264], rax
    mov eax, [rbp + -24]
    lea rcx, [rel sd_start]
    mov rdx, [rbp + -264]
    mov [rcx + rdx*4], eax
    mov rax, [rel sd_buf]
    mov [rbp + -272], rax
    mov eax, 2
    mov [rbp + -280], rax
    mov rax, [rbp + -280]
    lea rdx, [rel co]
    mov ecx, [rdx + rax*4]
    mov [rbp + -288], rcx
    mov rcx, [rbp + -272]
    mov edx, [rbp + -24]
    mov r8d, [rbp + -288]
    sub rsp, 32
    xor eax, eax
    call str_save
    add rsp, 32
    mov [rbp + -296], rax
    mov eax, [rbp + -296]
    mov [rbp + -24], rax
    mov eax, [rel sd_n]
    mov [rbp + -304], rax
    mov eax, [rel sd_n]
    mov [rbp + -312], rax
    mov eax, 1
    mov [rbp + -320], rax
    mov eax, [rbp + -312]
    mov ecx, [rbp + -320]
    add eax, ecx
    mov [rbp + -328], rax
    mov eax, [rbp + -328]
    mov [rel sd_n], eax
    jmp load_students_BB3
    load_students_BB7:
    jmp load_students_BB5

section .data
load_students_str_0 db 114, 0
