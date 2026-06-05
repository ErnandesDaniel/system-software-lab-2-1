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
global str_save
    str_save:
    push rbp
    mov rbp, rsp
    sub rsp, 144
    mov [rbp + -40], rcx
    mov [rbp + -8], rdx
    mov [rbp + -16], r8
    str_save_BB0:
    mov eax, [rbp + -8]
    mov [rbp + -32], rax
    jmp str_save_BB1
    str_save_BB1:
    mov eax, 1
    mov [rbp + -48], rax
    mov eax, [rbp + -48]
    test eax, eax
    jne str_save_BB2
    jmp str_save_BB3
    str_save_BB2:
    mov rax, [rel row_buf]
    mov [rbp + -56], rax
    mov rcx, [rbp + -56]
    mov edx, [rbp + -16]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -64], rax
    mov eax, [rbp + -64]
    mov [rbp + -24], rax
    mov rcx, [rbp + -40]
    mov edx, [rbp + -32]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -72], rax
    mov rcx, [rbp + -40]
    mov edx, [rbp + -32]
    mov r8d, [rbp + -24]
    mov [rcx + rdx], r8b
    mov eax, 0
    mov [rbp + -80], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -80]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -88], rax
    mov eax, [rbp + -88]
    test eax, eax
    jne str_save_BB5
    jmp str_save_BB4
    str_save_BB3:
    mov eax, 1
    mov [rbp + -128], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -128]
    add eax, ecx
    mov [rbp + -136], rax
    mov eax, [rbp + -136]
    leave
    ret
    str_save_BB4:
    mov eax, 1
    mov [rbp + -96], rax
    mov eax, [rbp + -16]
    mov ecx, [rbp + -96]
    add eax, ecx
    mov [rbp + -104], rax
    mov eax, [rbp + -104]
    mov [rbp + -16], rax
    mov eax, 1
    mov [rbp + -112], rax
    mov eax, [rbp + -32]
    mov ecx, [rbp + -112]
    add eax, ecx
    mov [rbp + -120], rax
    mov eax, [rbp + -120]
    mov [rbp + -32], rax
    jmp str_save_BB1
    str_save_BB5:
    jmp str_save_BB3
