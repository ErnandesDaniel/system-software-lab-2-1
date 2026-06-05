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
global print_str

extern putchar

    print_str:
    push rbp
    mov rbp, rsp
    sub rsp, 96
    mov [rbp + -24], rcx
    mov [rbp + -8], rdx
    print_str_BB0:
    mov eax, [rbp + -8]
    mov [rbp + -16], rax
    jmp print_str_BB1
    print_str_BB1:
    mov eax, 1
    mov [rbp + -40], rax
    mov eax, [rbp + -40]
    test eax, eax
    jne print_str_BB2
    jmp print_str_BB3
    print_str_BB2:
    mov rcx, [rbp + -24]
    mov edx, [rbp + -16]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -48], rax
    mov eax, [rbp + -48]
    mov [rbp + -32], rax
    mov eax, 0
    mov [rbp + -56], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -56]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -64], rax
    mov eax, [rbp + -64]
    test eax, eax
    jne print_str_BB5
    jmp print_str_BB4
    print_str_BB3:
    leave
    ret
    print_str_BB4:
    mov ecx, [rbp + -32]
    sub rsp, 32
    xor eax, eax
    call putchar
    add rsp, 32
    mov [rbp + -72], rax
    mov eax, 1
    mov [rbp + -80], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -80]
    add eax, ecx
    mov [rbp + -88], rax
    mov eax, [rbp + -88]
    mov [rbp + -16], rax
    jmp print_str_BB1
    print_str_BB5:
    jmp print_str_BB3
