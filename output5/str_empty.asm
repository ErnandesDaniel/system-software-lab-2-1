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
global str_empty
    str_empty:
    push rbp
    mov rbp, rsp
    sub rsp, 64
    mov [rbp + -8], rcx
    mov [rbp + -16], rdx
    str_empty_BB0:
    mov rcx, [rbp + -8]
    mov edx, [rbp + -16]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -24], rax
    mov eax, 0
    mov [rbp + -32], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -32]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -40], rax
    mov eax, [rbp + -40]
    test eax, eax
    jne str_empty_BB2
    jmp str_empty_BB1
    str_empty_BB1:
    mov eax, 0
    mov [rbp + -56], rax
    mov eax, [rbp + -56]
    leave
    ret
    str_empty_BB2:
    mov eax, 1
    mov [rbp + -48], rax
    mov eax, [rbp + -48]
    leave
    ret
