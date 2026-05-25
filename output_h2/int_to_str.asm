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

global int_to_str
extern buf_clear
extern choose_digit
extern int_to_str
extern strcat

int_to_str:
    push rbp
    mov rbp, rsp
    sub rsp, 160
    mov [rbp + -24], rcx
    mov [rbp + -32], rdx
BB_0:
    mov rax, [rbp + -24]
    mov rcx, rax
    sub rsp, 32
    call buf_clear
    add rsp, 32
    mov eax, 0
    mov [rbp + -112], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -112]
    cmp eax, ebx
    setl al
    movzx eax, al
    mov [rbp + -120], eax
    mov eax, [rbp + -120]
    test eax, eax
    jne BB_1
    jmp BB_2
BB_1:
    lea rax, [int_to_str_str_0]
    mov [rbp + -80], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -80]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -88], rax
    mov eax, 0
    mov [rbp + -96], eax
    mov eax, [rbp + -96]
    mov ebx, [rbp + -32]
    sub eax, ebx
    mov [rbp + -104], eax
    mov eax, [rbp + -104]
    mov [rbp + -32], eax
    jmp BB_3
BB_2:
    jmp BB_3
BB_3:
    mov eax, 10
    mov [rbp + -144], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -144]
    cmp eax, ebx
    setge al
    movzx eax, al
    mov [rbp + -152], eax
    mov eax, [rbp + -152]
    test eax, eax
    jne BB_4
    jmp BB_5
BB_4:
    mov eax, 10
    mov [rbp + -128], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -128]
    cdq
    idiv ebx
    mov [rbp + -136], eax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov edx, [rbp + -136]
    sub rsp, 32
    call int_to_str
    add rsp, 32
    jmp BB_6
BB_5:
    jmp BB_6
BB_6:
    mov eax, 10
    mov [rbp + -40], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -40]
    cdq
    idiv ebx
    mov [rbp + -48], eax
    mov eax, 10
    mov [rbp + -56], eax
    mov eax, [rbp + -48]
    mov ebx, [rbp + -56]
    imul eax, ebx
    mov [rbp + -64], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -64]
    sub eax, ebx
    mov [rbp + -72], eax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov edx, [rbp + -72]
    sub rsp, 32
    call choose_digit
    add rsp, 32
    leave
    ret

section .data
int_to_str_str_0 db 45, 0
