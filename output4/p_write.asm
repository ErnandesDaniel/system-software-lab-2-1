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
global p_write
    p_write:
    push rbp
    mov rbp, rsp
    sub rsp, 1200
    mov [rbp + -2048], rcx
    mov [rbp + -2056], rdx
    p_write_BB0:
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1032]
    mov [rbp + -2064], rax
    mov eax, 256
    mov [rbp + -2072], rax
    mov eax, [rbp + -2064]
    mov ecx, [rbp + -2072]
    cmp eax, ecx
    setl al
    movzx eax, al
    mov [rbp + -2080], rax
    mov eax, [rbp + -2080]
    test eax, eax
    jne p_write_BB2
    jmp p_write_BB1
    p_write_BB1:
    mov eax, 1
    mov [rbp + -2200], rax
    mov eax, [rbp + -2200]
    neg eax
    mov [rbp + -2208], rax
    mov eax, [rbp + -2208]
    leave
    ret
    p_write_BB2:
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1028]
    mov [rbp + -2088], rax
    mov rax, [rbp + -2088]
    lea rdx, [rbp + -2048]
    imul rax, 4
    mov ecx, [rdx + rax]
    mov [rbp + -2096], rcx
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1028]
    mov [rbp + -2104], rax
    mov eax, [rbp + -2056]
    lea rcx, [rbp + -2048]
    mov rdx, [rbp + -2104]
    mov [rcx + rdx*4], eax
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1028]
    mov [rbp + -2112], rax
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1028]
    mov [rbp + -2120], rax
    mov eax, 1
    mov [rbp + -2128], rax
    mov eax, [rbp + -2120]
    mov ecx, [rbp + -2128]
    add eax, ecx
    mov [rbp + -2136], rax
    mov eax, 256
    mov [rbp + -2144], rax
    mov eax, [rbp + -2136]
    mov ebx, [rbp + -2144]
    cdq
    idiv ebx
    mov [rbp + -2152], rdx
    mov eax, [rbp + -2152]
    lea rcx, [rbp + -2048]
    mov [rcx + 1028], eax
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1032]
    mov [rbp + -2160], rax
    lea rcx, [rbp + -2048]
    mov eax, [rcx + 1032]
    mov [rbp + -2168], rax
    mov eax, 1
    mov [rbp + -2176], rax
    mov eax, [rbp + -2168]
    mov ecx, [rbp + -2176]
    add eax, ecx
    mov [rbp + -2184], rax
    mov eax, [rbp + -2184]
    lea rcx, [rbp + -2048]
    mov [rcx + 1032], eax
    mov eax, 0
    mov [rbp + -2192], rax
    mov eax, [rbp + -2192]
    leave
    ret
