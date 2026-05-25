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

global choose_digit
extern strcat

choose_digit:
    push rbp
    mov rbp, rsp
    sub rsp, 368
    mov [rbp + -24], rcx
    mov [rbp + -32], rdx
BB_0:
    mov eax, 0
    mov [rbp + -56], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -56]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -64], eax
    mov eax, [rbp + -64]
    test eax, eax
    jne BB_1
    jmp BB_2
BB_1:
    mov rax, [rel d0]
    mov [rbp + -40], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -40]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -48], rax
    jmp BB_3
BB_2:
    jmp BB_3
BB_3:
    mov eax, 1
    mov [rbp + -88], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -88]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -96], eax
    mov eax, [rbp + -96]
    test eax, eax
    jne BB_4
    jmp BB_5
BB_4:
    mov rax, [rel d1]
    mov [rbp + -72], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -72]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -80], rax
    jmp BB_6
BB_5:
    jmp BB_6
BB_6:
    mov eax, 2
    mov [rbp + -120], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -120]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -128], eax
    mov eax, [rbp + -128]
    test eax, eax
    jne BB_7
    jmp BB_8
BB_7:
    mov rax, [rel d2]
    mov [rbp + -104], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -104]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -112], rax
    jmp BB_9
BB_8:
    jmp BB_9
BB_9:
    mov eax, 3
    mov [rbp + -152], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -152]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -160], eax
    mov eax, [rbp + -160]
    test eax, eax
    jne BB_10
    jmp BB_11
BB_10:
    mov rax, [rel d3]
    mov [rbp + -136], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -136]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -144], rax
    jmp BB_12
BB_11:
    jmp BB_12
BB_12:
    mov eax, 4
    mov [rbp + -184], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -184]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -192], eax
    mov eax, [rbp + -192]
    test eax, eax
    jne BB_13
    jmp BB_14
BB_13:
    mov rax, [rel d4]
    mov [rbp + -168], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -168]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -176], rax
    jmp BB_15
BB_14:
    jmp BB_15
BB_15:
    mov eax, 5
    mov [rbp + -216], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -216]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -224], eax
    mov eax, [rbp + -224]
    test eax, eax
    jne BB_16
    jmp BB_17
BB_16:
    mov rax, [rel d5]
    mov [rbp + -200], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -200]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -208], rax
    jmp BB_18
BB_17:
    jmp BB_18
BB_18:
    mov eax, 6
    mov [rbp + -248], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -248]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -256], eax
    mov eax, [rbp + -256]
    test eax, eax
    jne BB_19
    jmp BB_20
BB_19:
    mov rax, [rel d6]
    mov [rbp + -232], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -232]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -240], rax
    jmp BB_21
BB_20:
    jmp BB_21
BB_21:
    mov eax, 7
    mov [rbp + -280], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -280]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -288], eax
    mov eax, [rbp + -288]
    test eax, eax
    jne BB_22
    jmp BB_23
BB_22:
    mov rax, [rel d7]
    mov [rbp + -264], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -264]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -272], rax
    jmp BB_24
BB_23:
    jmp BB_24
BB_24:
    mov eax, 8
    mov [rbp + -312], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -312]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -320], eax
    mov eax, [rbp + -320]
    test eax, eax
    jne BB_25
    jmp BB_26
BB_25:
    mov rax, [rel d8]
    mov [rbp + -296], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -296]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -304], rax
    jmp BB_27
BB_26:
    jmp BB_27
BB_27:
    mov eax, 9
    mov [rbp + -344], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -344]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -352], eax
    mov eax, [rbp + -352]
    test eax, eax
    jne BB_28
    jmp BB_29
BB_28:
    mov rax, [rel d9]
    mov [rbp + -328], rax
    mov rax, [rbp + -24]
    mov rcx, rax
    mov rax, [rbp + -328]
    mov rdx, rax
    sub rsp, 32
    call strcat
    add rsp, 32
    mov [rbp + -336], rax
    jmp BB_30
BB_29:
    jmp BB_30
BB_30:
    leave
    ret
