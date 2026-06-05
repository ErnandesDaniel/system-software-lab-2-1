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
global str_gt_lit
    str_gt_lit:
    push rbp
    mov rbp, rsp
    sub rsp, 224
    mov [rbp + -16], rcx
    mov [rbp + -40], rdx
    mov [rbp + -32], r8
    str_gt_lit_BB0:
    mov eax, 0
    mov [rbp + -56], rax
    mov eax, [rbp + -56]
    mov [rbp + -48], rax
    jmp str_gt_lit_BB1
    str_gt_lit_BB1:
    mov eax, 1
    mov [rbp + -64], rax
    mov eax, [rbp + -64]
    test eax, eax
    jne str_gt_lit_BB2
    jmp str_gt_lit_BB3
    str_gt_lit_BB2:
    mov eax, [rbp + -40]
    mov ecx, [rbp + -48]
    add eax, ecx
    mov [rbp + -72], rax
    mov rcx, [rbp + -16]
    mov edx, [rbp + -72]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -80], rax
    mov eax, [rbp + -80]
    mov [rbp + -24], rax
    mov rcx, [rbp + -32]
    mov edx, [rbp + -48]
    movzx eax, byte [rcx + rdx]
    mov [rbp + -88], rax
    mov eax, [rbp + -88]
    mov [rbp + -8], rax
    mov eax, 0
    mov [rbp + -96], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -96]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -104], rax
    mov eax, [rbp + -104]
    test eax, eax
    jne str_gt_lit_BB5
    jmp str_gt_lit_BB6
    str_gt_lit_BB3:
    str_gt_lit_BB4:
    mov eax, 0
    mov [rbp + -160], rax
    mov eax, [rbp + -24]
    mov ecx, [rbp + -160]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -168], rax
    mov eax, [rbp + -168]
    test eax, eax
    jne str_gt_lit_BB10
    jmp str_gt_lit_BB9
    str_gt_lit_BB5:
    mov eax, 0
    mov [rbp + -112], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -112]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -120], rax
    mov eax, [rbp + -120]
    mov [rbp + -128], rax
    jmp str_gt_lit_BB7
    str_gt_lit_BB6:
    mov eax, 0
    mov [rbp + -128], rax
    jmp str_gt_lit_BB7
    str_gt_lit_BB7:
    mov eax, [rbp + -128]
    test eax, eax
    jne str_gt_lit_BB8
    jmp str_gt_lit_BB4
    str_gt_lit_BB8:
    mov eax, 0
    mov [rbp + -136], rax
    mov eax, [rbp + -136]
    leave
    ret
    str_gt_lit_BB9:
    mov eax, 0
    mov [rbp + -184], rax
    mov eax, [rbp + -8]
    mov ecx, [rbp + -184]
    cmp eax, ecx
    sete al
    movzx eax, al
    mov [rbp + -192], rax
    mov eax, [rbp + -192]
    test eax, eax
    jne str_gt_lit_BB12
    jmp str_gt_lit_BB11
    str_gt_lit_BB10:
    mov eax, 1
    mov [rbp + -144], rax
    mov eax, [rbp + -144]
    neg eax
    mov [rbp + -152], rax
    mov eax, [rbp + -152]
    leave
    ret
    str_gt_lit_BB11:
    mov eax, [rbp + -24]
    mov ecx, [rbp + -8]
    cmp eax, ecx
    setne al
    movzx eax, al
    mov [rbp + -208], rax
    mov eax, [rbp + -208]
    test eax, eax
    jne str_gt_lit_BB14
    jmp str_gt_lit_BB13
    str_gt_lit_BB12:
    mov eax, 1
    mov [rbp + -176], rax
    mov eax, [rbp + -176]
    leave
    ret
    str_gt_lit_BB13:
    mov eax, 1
    mov [rbp + -216], rax
    mov eax, [rbp + -48]
    mov ecx, [rbp + -216]
    add eax, ecx
    mov [rbp + -224], rax
    mov eax, [rbp + -224]
    mov [rbp + -48], rax
    jmp str_gt_lit_BB1
    str_gt_lit_BB14:
    mov eax, [rbp + -24]
    mov ecx, [rbp + -8]
    sub eax, ecx
    mov [rbp + -200], rax
    mov eax, [rbp + -200]
    leave
    ret
