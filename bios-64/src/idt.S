        .code64
        .section .stub, "ax"

        .global stub_de
        .global stub_br
        .global stub_ud
        .global stub_nm
        .global stub_df
        .global stub_ts
        .global stub_np
        .global stub_ss
        .global stub_gp
        .global stub_pf
        .global stub_mf
        .global stub_ac
        .global stub_mc
        .global stub_xm
        .global stub_ve
        .global stub_cp
        .global stub_hv
        .global stub_vc
        .global stub_sx

stub_de:
        pushq	$0      // dummy error code
        pushq	$0	// division error
        jmp     stub_common

stub_br:
        pushq	$0      // dummy error code
        pushq	$5	// bound range exceeded
        jmp     stub_common

stub_ud:
        pushq	$0      // dummy error code
        pushq	$6	// invalid opcode
        jmp     stub_common

stub_nm:
        pushq	$0      // dummy error code
        pushq	$7	// device not available
        jmp     stub_common

stub_df:
        pushq	$8	// double fault
        jmp     stub_common

stub_ts:
        pushq	$10	// invalid tss
        jmp     stub_common

stub_np:
        pushq	$11	// segment not present
        jmp     stub_common

stub_ss:
        pushq	$12	// stack segmment fault
        jmp     stub_common

stub_gp:
        pushq	$13	// general protection fault
        jmp     stub_common

stub_pf:
        pushq	$14	// page fault
        jmp     stub_common

stub_mf:
        pushq	$0      // dummy error code
        pushq	$16	// floating point exception
        jmp     stub_common

stub_ac:
        pushq	$17	// alignment check
        jmp     stub_common

stub_mc:
        pushq	$0      // dummy error code
        pushq	$18	// machine check
        jmp     stub_common

stub_xm:
        pushq	$0      // dummy error code
        pushq	$19	// SMID floating point exception
        jmp     stub_common

stub_ve:
        pushq	$0      // dummy error code
        pushq	$20	// virtualization exception
        jmp     stub_common

stub_cp:
        pushq	$21	// control protection exception
        jmp     stub_common

stub_hv:
        pushq	$0      // dummy error code
        pushq	$28	// hypervisor injection exception
        jmp     stub_common

stub_vc:
        pushq	$29	// vmm communication exception
        jmp     stub_common

stub_sx:
        pushq	$30	// security exception
        jmp     stub_common

stub_common:
        push    %rax
        push    %rbx
        push    %rcx
        push    %rdx
        push    %rbp
        push    %rdi
        push    %rsi
        push    %r8
        push    %r9
        push    %r10
        push    %r11
        push    %r12
        push    %r13
        push    %r14
        push    %r15

        mov     %rsp, %rdi
        call    exception_handler

        pop     %r15
        pop     %r14
        pop     %r13
        pop     %r12
        pop     %r11
        pop     %r10
        pop     %r9
        pop     %r8
        pop     %rsi
        pop     %rdi
        pop     %rbp
        pop     %rdx
        pop     %rcx
        pop     %rbx
        pop     %rax

        add     $16, %rsp
        iretq
