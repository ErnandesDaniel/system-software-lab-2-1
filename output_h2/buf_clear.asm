extern tv_id
extern tv_name
extern tv_count
extern v_id
extern v_type_id
extern v_person_id
extern v_date
extern v_mark
extern v_count
extern p_id
extern p_surname
extern p_name
extern p_patronymic
extern p_birthday
extern p_count
extern o_person_id
extern o_nzk
extern o_form
extern o_department
extern o_fac
extern o_course
extern o_count
extern st_person_id
extern st_group
extern st_start
extern st_order
extern st_state
extern st_count
extern gp_plan_id
extern gp_group
extern gp_dept
extern gp_count
extern buf
extern tmpbuf
extern d0
extern d1
extern d2
extern d3
extern d4
extern d5
extern d6
extern d7
extern d8
extern d9
bits 64
default rel
section .text

bits 64
default rel
section .text

global buf_clear
extern strcpy

buf_clear:
    push rbp
    mov rbp, rsp
    sub rsp, 48
    mov [rbp + -16], rcx
BB_0:
    lea rax, [buf_clear_str_0]
    mov [rbp + -24], rax
    mov rax, [rbp + -16]
    mov rcx, rax
    mov rax, [rbp + -24]
    mov rdx, rax
    sub rsp, 32
    call strcpy
    add rsp, 32
    mov [rbp + -32], rax
    leave
    ret

section .data
buf_clear_str_0 db 0, 0
