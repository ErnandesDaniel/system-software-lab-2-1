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
global p_read
    p_read:
    push rbp
    mov rbp, rsp
    sub rsp, 1184
    mov [rbp + -2056], rcx
    p_read_BB0:
    lea rcx, [rbp + -2056]
    mov eax, [rcx + 1032]
    mov [rbp + -2064], rax
    mov eax, 0
    mov [rbp + -2072], rax
    mov eax, [rbp + -2064]
    mov ecx, [rbp + -2072]
    cmp eax, ecx
    setle al
    movzx eax, al
    mov [rbp + -2080], rax
    mov eax, [rbp + -2080]
    test eax, eax
    jne p_read_BB2
    jmp p_read_BB1
    p_read_BB1:
    lea rcx, [rbp + -2056]
    mov eax, [rcx + 1024]
    mov [rbp + -2104], rax
    mov rax, [rbp + -2104]
    lea rdx, [rbp + -2056]
    imul rax, 4
    mov ecx, [rdx + rax]
    mov [rbp + -2112], rcx
    mov eax, [rbp + -2112]
    mov [rbp + -8], rax
    lea rcx, [rbp + -2056]
    mov eax, [rcx + 1024]
    mov [rbp + -2120], rax
    lea rcx, [rbp + -2056]
    mov eax, [rcx + 1024]
    mov [rbp + -2128], rax
    mov eax, 1
    mov [rbp + -2136], rax
    mov eax, [rbp + -2128]
    mov ecx, [rbp + -2136]
    add eax, ecx
    mov [rbp + -2144], rax
    mov eax, 256
    mov [rbp + -2152], rax
    mov eax, [rbp + -2144]
    mov ebx, [rbp + -2152]
    cdq
    idiv ebx
    mov [rbp + -2160], rdx
    mov eax, [rbp + -2160]
    lea rcx, [rbp + -2056]
    mov [rcx + 1024], eax
    lea rcx, [rbp + -2056]
    mov eax, [rcx + 1032]
    mov [rbp + -2168], rax
    lea rcx, [rbp + -2056]
    mov eax, [rcx + 1032]
    mov [rbp + -2176], rax
    mov eax, 1
    mov [rbp + -2184], rax
    mov eax, [rbp + -2176]
    mov ecx, [rbp + -2184]
    sub eax, ecx
    mov [rbp + -2192], rax
    mov eax, [rbp + -2192]
    lea rcx, [rbp + -2056]
    mov [rcx + 1032], eax
    mov eax, [rbp + -8]
    leave
    ret
    p_read_BB2:
    mov eax, 1
    mov [rbp + -2088], rax
    mov eax, [rbp + -2088]
    neg eax
    mov [rbp + -2096], rax
    mov eax, [rbp + -2096]
    leave
    ret
