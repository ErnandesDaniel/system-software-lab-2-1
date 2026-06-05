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
global main

extern load_group_plans
extern load_people
extern load_students
extern load_studies
extern load_types
extern load_vedomosti
extern malloc
extern printf
extern puts
extern q1
extern q2
extern q3
extern q4
extern q5
extern q6
extern q7

    main:
    push rbp
    mov rbp, rsp
    sub rsp, 352
    main_BB0:
    mov rax, [rel row_buf]
    mov [rbp + -8], rax
    mov eax, 10000
    mov [rbp + -16], rax
    mov ecx, [rbp + -16]
    sub rsp, 32
    xor eax, eax
    call malloc
    add rsp, 32
    mov [rbp + -24], rax
    mov rax, [rbp + -24]
    mov [rel row_buf], rax
    mov rax, [rel row_buf]
    mov [rbp + -32], rax
    mov eax, 0
    mov [rbp + -40], rax
    mov rax, [rbp + -32]
    mov ecx, [rbp + -40]
    cmp rax, rcx
    sete al
    movzx eax, al
    mov [rbp + -48], rax
    mov eax, [rbp + -48]
    test eax, eax
    jne main_BB2
    jmp main_BB1
    main_BB1:
    lea rax, [main_str_0]
    mov [rbp + -80], rax
    mov rcx, [rbp + -80]
    sub rsp, 32
    xor eax, eax
    call puts
    add rsp, 32
    mov [rbp + -88], rax
    lea rax, [main_str_1]
    mov [rbp + -96], rax
    mov rcx, [rbp + -96]
    sub rsp, 32
    xor eax, eax
    call load_types
    add rsp, 32
    mov [rbp + -104], rax
    lea rax, [main_str_2]
    mov [rbp + -112], rax
    mov rcx, [rbp + -112]
    sub rsp, 32
    xor eax, eax
    call load_people
    add rsp, 32
    mov [rbp + -120], rax
    lea rax, [main_str_3]
    mov [rbp + -128], rax
    mov rcx, [rbp + -128]
    sub rsp, 32
    xor eax, eax
    call load_studies
    add rsp, 32
    mov [rbp + -136], rax
    lea rax, [main_str_4]
    mov [rbp + -144], rax
    mov rcx, [rbp + -144]
    sub rsp, 32
    xor eax, eax
    call load_students
    add rsp, 32
    mov [rbp + -152], rax
    lea rax, [main_str_5]
    mov [rbp + -160], rax
    mov rcx, [rbp + -160]
    sub rsp, 32
    xor eax, eax
    call load_vedomosti
    add rsp, 32
    mov [rbp + -168], rax
    lea rax, [main_str_6]
    mov [rbp + -176], rax
    mov rcx, [rbp + -176]
    sub rsp, 32
    xor eax, eax
    call load_group_plans
    add rsp, 32
    mov [rbp + -184], rax
    lea rax, [main_str_7]
    mov [rbp + -192], rax
    mov eax, [rel tv_n]
    mov [rbp + -200], rax
    mov rcx, [rbp + -192]
    mov edx, [rbp + -200]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -208], rax
    lea rax, [main_str_8]
    mov [rbp + -216], rax
    mov eax, [rel pp_n]
    mov [rbp + -224], rax
    mov rcx, [rbp + -216]
    mov edx, [rbp + -224]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -232], rax
    lea rax, [main_str_9]
    mov [rbp + -240], rax
    mov eax, [rel st_n]
    mov [rbp + -248], rax
    mov rcx, [rbp + -240]
    mov edx, [rbp + -248]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -256], rax
    lea rax, [main_str_10]
    mov [rbp + -264], rax
    mov eax, [rel sd_n]
    mov [rbp + -272], rax
    mov rcx, [rbp + -264]
    mov edx, [rbp + -272]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -280], rax
    lea rax, [main_str_11]
    mov [rbp + -288], rax
    mov eax, [rel vv_n]
    mov [rbp + -296], rax
    mov rcx, [rbp + -288]
    mov edx, [rbp + -296]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -304], rax
    lea rax, [main_str_12]
    mov [rbp + -312], rax
    mov eax, [rel gp_n]
    mov [rbp + -320], rax
    mov rcx, [rbp + -312]
    mov edx, [rbp + -320]
    sub rsp, 32
    xor eax, eax
    call printf
    add rsp, 32
    mov [rbp + -328], rax
    sub rsp, 32
    xor eax, eax
    call q1
    add rsp, 32
    sub rsp, 32
    xor eax, eax
    call q2
    add rsp, 32
    sub rsp, 32
    xor eax, eax
    call q3
    add rsp, 32
    sub rsp, 32
    xor eax, eax
    call q4
    add rsp, 32
    sub rsp, 32
    xor eax, eax
    call q5
    add rsp, 32
    sub rsp, 32
    xor eax, eax
    call q6
    add rsp, 32
    sub rsp, 32
    xor eax, eax
    call q7
    add rsp, 32
    lea rax, [main_str_13]
    mov [rbp + -336], rax
    mov rcx, [rbp + -336]
    sub rsp, 32
    xor eax, eax
    call puts
    add rsp, 32
    mov [rbp + -344], rax
    mov eax, 0
    mov [rbp + -352], rax
    mov eax, [rbp + -352]
    leave
    ret
    main_BB2:
    lea rax, [main_str_14]
    mov [rbp + -56], rax
    mov rcx, [rbp + -56]
    sub rsp, 32
    xor eax, eax
    call puts
    add rsp, 32
    mov [rbp + -64], rax
    mov eax, 1
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    leave
    ret

section .data
main_str_0 db 61, 61, 61, 32, 76, 97, 98, 32, 50, 58, 32, 77, 97, 112, 45, 82, 101, 100, 117, 99, 101, 32, 80, 105, 112, 101, 108, 105, 110, 101, 32, 61, 61, 61, 0
main_str_1 db 108, 97, 98, 115, 45, 101, 120, 97, 109, 112, 108, 101, 115, 47, 115, 121, 115, 116, 101, 109, 45, 112, 114, 111, 103, 114, 97, 109, 109, 115, 47, 108, 97, 98, 45, 50, 47, 99, 115, 118, 45, 100, 97, 116, 97, 47, 116, 121, 112, 101, 115, 95, 118, 101, 100, 111, 109, 111, 115, 116, 101, 105, 46, 99, 115, 118, 0
main_str_2 db 108, 97, 98, 115, 45, 101, 120, 97, 109, 112, 108, 101, 115, 47, 115, 121, 115, 116, 101, 109, 45, 112, 114, 111, 103, 114, 97, 109, 109, 115, 47, 108, 97, 98, 45, 50, 47, 99, 115, 118, 45, 100, 97, 116, 97, 47, 112, 101, 111, 112, 108, 101, 46, 99, 115, 118, 0
main_str_3 db 108, 97, 98, 115, 45, 101, 120, 97, 109, 112, 108, 101, 115, 47, 115, 121, 115, 116, 101, 109, 45, 112, 114, 111, 103, 114, 97, 109, 109, 115, 47, 108, 97, 98, 45, 50, 47, 99, 115, 118, 45, 100, 97, 116, 97, 47, 115, 116, 117, 100, 105, 101, 115, 46, 99, 115, 118, 0
main_str_4 db 108, 97, 98, 115, 45, 101, 120, 97, 109, 112, 108, 101, 115, 47, 115, 121, 115, 116, 101, 109, 45, 112, 114, 111, 103, 114, 97, 109, 109, 115, 47, 108, 97, 98, 45, 50, 47, 99, 115, 118, 45, 100, 97, 116, 97, 47, 115, 116, 117, 100, 101, 110, 116, 115, 46, 99, 115, 118, 0
main_str_5 db 108, 97, 98, 115, 45, 101, 120, 97, 109, 112, 108, 101, 115, 47, 115, 121, 115, 116, 101, 109, 45, 112, 114, 111, 103, 114, 97, 109, 109, 115, 47, 108, 97, 98, 45, 50, 47, 99, 115, 118, 45, 100, 97, 116, 97, 47, 118, 101, 100, 111, 109, 111, 115, 116, 105, 46, 99, 115, 118, 0
main_str_6 db 108, 97, 98, 115, 45, 101, 120, 97, 109, 112, 108, 101, 115, 47, 115, 121, 115, 116, 101, 109, 45, 112, 114, 111, 103, 114, 97, 109, 109, 115, 47, 108, 97, 98, 45, 50, 47, 99, 115, 118, 45, 100, 97, 116, 97, 47, 103, 114, 111, 117, 112, 95, 112, 108, 97, 110, 115, 46, 99, 115, 118, 0
main_str_7 db 84, 121, 112, 101, 115, 58, 32, 37, 100, 10, 0
main_str_8 db 80, 101, 111, 112, 108, 101, 58, 32, 37, 100, 10, 0
main_str_9 db 83, 116, 117, 100, 105, 101, 115, 58, 32, 37, 100, 10, 0
main_str_10 db 83, 116, 117, 100, 101, 110, 116, 115, 58, 32, 37, 100, 10, 0
main_str_11 db 86, 101, 100, 111, 109, 111, 115, 116, 105, 58, 32, 37, 100, 10, 0
main_str_12 db 80, 108, 97, 110, 115, 58, 32, 37, 100, 10, 0
main_str_13 db 61, 61, 61, 32, 68, 111, 110, 101, 32, 61, 61, 61, 0
main_str_14 db 109, 97, 108, 108, 111, 99, 32, 102, 97, 105, 108, 101, 100, 0
