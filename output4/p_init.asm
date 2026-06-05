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
global p_init
    p_init:
    push rbp
    mov rbp, rsp
    sub rsp, 1088
    mov [rbp + -2048], rcx
    p_init_BB0:
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1024]
    mov [rbp + -2056], rax
    mov eax, 0
    mov [rbp + -2064], rax
    mov eax, [rbp + -2064]
    lea rcx, [rbp + -2048]
    mov [rcx + 1024], eax
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1028]
    mov [rbp + -2072], rax
    mov eax, 0
    mov [rbp + -2080], rax
    mov eax, [rbp + -2080]
    lea rcx, [rbp + -2048]
    mov [rcx + 1028], eax
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1032]
    mov [rbp + -2088], rax
    mov eax, 0
    mov [rbp + -2096], rax
    mov eax, [rbp + -2096]
    lea rcx, [rbp + -2048]
    mov [rcx + 1032], eax
    leave
    ret
