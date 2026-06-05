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
global str_eq_lit
    str_eq_lit:
    push rbp
    mov rbp, rsp
    sub rsp, 144
    mov [rbp + -40], rcx
    mov [rbp + -8], rdx
    mov [rbp + -24], r8
    str_eq_lit_BB0:
    mov eax, 0
    mov [rbp + -56], rax
    mov eax, [rbp + -56]
    mov [rbp + -16], rax
    jmp str_eq_lit_BB1
    str_eq_lit_BB1:
    mov eax, 1
    mov [rbp + -64], rax
    mov eax, [rbp + -64]
    test eax, eax
    jne str_eq_lit_BB2
    jmp str_eq_lit_BB3
    str_eq_lit_BB2:
    mov eax, [rbp + -8]
    mov ecx, [rbp + -16]
    add eax, ecx
    mov [rbp + -80], rax
    mov rcx, [rbp + -40]
    mov edx, [rbp + -80]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -88], rax
    mov eax, [rbp + -88]
    mov [rbp + -48], rax
    mov rcx, [rbp + -24]
    mov edx, [rbp + -16]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -96], rax
    mov eax, [rbp + -96]
    mov [rbp + -32], rax
    mov eax, [rbp + -48]
    mov ecx, [rbp + -32]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -104], rax
    mov eax, [rbp + -104]
    test eax, eax
    jne str_eq_lit_BB5
    jmp str_eq_lit_BB4
    str_eq_lit_BB3:
    str_eq_lit_BB4:
    mov eax, 0
    mov [rbp + -120], rax
    mov eax, [rbp + -48]
    mov ecx, [rbp + -120]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -128], rax
    mov eax, [rbp + -128]
    test eax, eax
    jne str_eq_lit_BB7
    jmp str_eq_lit_BB6
    str_eq_lit_BB5:
    mov eax, 0
    mov [rbp + -72], rax
    mov eax, [rbp + -72]
    leave
    ret
    str_eq_lit_BB6:
    mov eax, 1
    mov [rbp + -136], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -136]
    add eax, ecx
    mov [rbp + -144], rax
    mov eax, [rbp + -144]
    mov [rbp + -16], rax
    jmp str_eq_lit_BB1
    str_eq_lit_BB7:
    mov eax, 1
    mov [rbp + -112], rax
    mov eax, [rbp + -112]
    leave
    ret
