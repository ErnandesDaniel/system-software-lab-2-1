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

global main
extern puts

main:
    push rbp
    mov rbp, rsp
    sub rsp, 32
BB_0:
    lea rax, [main_str_0]
    mov [rbp + -8], rax
    mov rax, [rbp + -8]
    mov rcx, rax
    sub rsp, 32
    call puts
    add rsp, 32
    mov [rbp + -16], eax
    mov eax, 0
    mov [rbp + -24], eax
    mov eax, [rbp + -24]
    leave
    ret

section .data
main_str_0 db 111, 107, 0
