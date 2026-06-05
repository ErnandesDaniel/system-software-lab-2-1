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
global col_drain

extern p_read

    col_drain:
    push rbp
    mov rbp, rsp
    sub rsp, 1088
    mov [rbp + -2048], rcx
    col_drain_BB0:
    jmp col_drain_BB1
    col_drain_BB1:
    mov eax, 1
    mov [rbp + -2056], rax
    mov eax, [rbp + -2056]
    test eax, eax
    jne col_drain_BB2
    jmp col_drain_BB3
    col_drain_BB2:
    lea rcx, [rbp + -2048]
    sub rsp, 32
    xor eax, eax
    call p_read
    add rsp, 32
    mov [rbp + -2064], rax
    mov eax, 1
    mov [rbp + -2072], rax
    mov eax, [rbp + -2072]
    neg eax
    mov [rbp + -2080], rax
    mov eax, [rbp + -2064]
    mov ecx, [rbp + -2080]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -2088], rax
    mov eax, [rbp + -2088]
    test eax, eax
    jne col_drain_BB5
    jmp col_drain_BB4
    col_drain_BB3:
    leave
    ret
    col_drain_BB4:
    jmp col_drain_BB1
    col_drain_BB5:
    leave
    ret
