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

global csv_field
extern str_get_byte
extern str_set_byte

csv_field:
    push rbp
    mov rbp, rsp
    sub rsp, 416
    mov [rbp + -56], rcx
    mov [rbp + -64], rdx
    mov [rbp + -72], r8
BB_0:
    mov eax, 0
    mov [rbp + -408], eax
    mov eax, [rbp + -408]
    mov [rbp + -24], eax
    jmp BB_1
BB_1:
    mov eax, 1
    mov [rbp + -80], eax
    mov eax, 1
    mov [rbp + -88], eax
    mov eax, [rbp + -80]
    mov ebx, [rbp + -88]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -96], eax
    mov eax, [rbp + -96]
    test eax, eax
    jne BB_2
    jmp BB_3
BB_2:
    mov eax, [rbp + -72]
    mov ebx, [rbp + -24]
    add eax, ebx
    mov [rbp + -128], eax
    mov rax, [rbp + -64]
    mov rcx, rax
    mov edx, [rbp + -128]
    sub rsp, 32
    call str_get_byte
    add rsp, 32
    mov [rbp + -136], eax
    mov eax, [rbp + -136]
    mov [rbp + -32], eax
    mov eax, 0
    mov [rbp + -144], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -144]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -152], eax
    mov eax, [rbp + -152]
    test eax, eax
    jne BB_4
    jmp BB_5
BB_3:
BB_4:
    mov eax, 0
    mov [rbp + -104], eax
    mov rax, [rbp + -56]
    mov rcx, rax
    mov edx, [rbp + -24]
    mov r8d, [rbp + -104]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -112], eax
    mov eax, [rbp + -72]
    mov ebx, [rbp + -24]
    add eax, ebx
    mov [rbp + -120], eax
    mov eax, [rbp + -120]
    leave
    ret
BB_5:
    jmp BB_6
BB_6:
    mov eax, 10
    mov [rbp + -184], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -184]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -192], eax
    mov eax, [rbp + -192]
    test eax, eax
    jne BB_7
    jmp BB_8
BB_7:
    mov eax, 0
    mov [rbp + -160], eax
    mov rax, [rbp + -56]
    mov rcx, rax
    mov edx, [rbp + -24]
    mov r8d, [rbp + -160]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -168], eax
    mov eax, [rbp + -72]
    mov ebx, [rbp + -24]
    add eax, ebx
    mov [rbp + -176], eax
    mov eax, [rbp + -176]
    leave
    ret
BB_8:
    jmp BB_9
BB_9:
    mov eax, 13
    mov [rbp + -312], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -312]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -320], eax
    mov eax, [rbp + -320]
    test eax, eax
    jne BB_10
    jmp BB_11
BB_10:
    mov eax, 0
    mov [rbp + -224], eax
    mov rax, [rbp + -56]
    mov rcx, rax
    mov edx, [rbp + -24]
    mov r8d, [rbp + -224]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -232], eax
    mov eax, [rbp + -72]
    mov ebx, [rbp + -24]
    add eax, ebx
    mov [rbp + -240], eax
    mov eax, 1
    mov [rbp + -248], eax
    mov eax, [rbp + -240]
    mov ebx, [rbp + -248]
    add eax, ebx
    mov [rbp + -256], eax
    mov rax, [rbp + -64]
    mov rcx, rax
    mov edx, [rbp + -256]
    sub rsp, 32
    call str_get_byte
    add rsp, 32
    mov [rbp + -264], eax
    mov eax, [rbp + -264]
    mov [rbp + -40], eax
    mov eax, 10
    mov [rbp + -272], eax
    mov eax, [rbp + -40]
    mov ebx, [rbp + -272]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -280], eax
    mov eax, [rbp + -280]
    test eax, eax
    jne BB_13
    jmp BB_14
BB_11:
    jmp BB_12
BB_12:
    mov eax, 44
    mov [rbp + -368], eax
    mov eax, [rbp + -32]
    mov ebx, [rbp + -368]
    cmp eax, ebx
    sete al
    movzx eax, al
    mov [rbp + -376], eax
    mov eax, [rbp + -376]
    test eax, eax
    jne BB_16
    jmp BB_17
BB_13:
    mov eax, [rbp + -72]
    mov ebx, [rbp + -24]
    add eax, ebx
    mov [rbp + -200], eax
    mov eax, 2
    mov [rbp + -208], eax
    mov eax, [rbp + -200]
    mov ebx, [rbp + -208]
    add eax, ebx
    mov [rbp + -216], eax
    mov eax, [rbp + -216]
    leave
    ret
BB_14:
    jmp BB_15
BB_15:
    mov eax, [rbp + -72]
    mov ebx, [rbp + -24]
    add eax, ebx
    mov [rbp + -288], eax
    mov eax, 1
    mov [rbp + -296], eax
    mov eax, [rbp + -288]
    mov ebx, [rbp + -296]
    add eax, ebx
    mov [rbp + -304], eax
    mov eax, [rbp + -304]
    leave
    ret
BB_16:
    mov eax, 0
    mov [rbp + -328], eax
    mov rax, [rbp + -56]
    mov rcx, rax
    mov edx, [rbp + -24]
    mov r8d, [rbp + -328]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -336], eax
    mov eax, [rbp + -72]
    mov ebx, [rbp + -24]
    add eax, ebx
    mov [rbp + -344], eax
    mov eax, 1
    mov [rbp + -352], eax
    mov eax, [rbp + -344]
    mov ebx, [rbp + -352]
    add eax, ebx
    mov [rbp + -360], eax
    mov eax, [rbp + -360]
    leave
    ret
BB_17:
    jmp BB_18
BB_18:
    mov rax, [rbp + -56]
    mov rcx, rax
    mov edx, [rbp + -24]
    mov r8d, [rbp + -32]
    sub rsp, 32
    call str_set_byte
    add rsp, 32
    mov [rbp + -384], eax
    mov eax, 1
    mov [rbp + -392], eax
    mov eax, [rbp + -24]
    mov ebx, [rbp + -392]
    add eax, ebx
    mov [rbp + -400], eax
    mov eax, [rbp + -400]
    mov [rbp + -24], eax
    jmp BB_1
