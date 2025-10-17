        .code64
        .section .stub, "ax"

stub_start:
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

        .section .idt, "a"
        .align  8

        .macro  idt_entry handler
        .word   (\handler & 0xFFFF)             // offset_low
        .word   0x18                            // code selector
        .byte   0                               // IST = 0
        .byte   0x8E                            // present, interrupt gate
        .word   ((\handler >> 16) & 0xFFFF)     // offset_mid
        .long   ((\handler >> 32) & 0xFFFFFFFF) // offset_high
        .long   0                               // reserved
        .endm

        .macro  idt_entry_empty
        .quad   0
        .quad   0
        .endm

idt_start:
        // linker script puts .stub section to 0xffffe000
        idt_entry (stub_de  - stub_start + 0xffffe000)
        idt_entry_empty
        idt_entry_empty
        idt_entry_empty
        idt_entry_empty
        idt_entry (stub_br  - stub_start + 0xffffe000)
        idt_entry (stub_ud  - stub_start + 0xffffe000)
        idt_entry (stub_nm  - stub_start + 0xffffe000)
        idt_entry (stub_df  - stub_start + 0xffffe000)
        idt_entry_empty
        idt_entry (stub_ts  - stub_start + 0xffffe000)
        idt_entry (stub_np  - stub_start + 0xffffe000)
        idt_entry (stub_ss  - stub_start + 0xffffe000)
        idt_entry (stub_gp  - stub_start + 0xffffe000)
        idt_entry (stub_pf  - stub_start + 0xffffe000)
        idt_entry_empty
        idt_entry (stub_mf  - stub_start + 0xffffe000)
        idt_entry (stub_ac  - stub_start + 0xffffe000)
        idt_entry (stub_mc  - stub_start + 0xffffe000)
        idt_entry (stub_xm  - stub_start + 0xffffe000)
        idt_entry (stub_ve  - stub_start + 0xffffe000)
        idt_entry (stub_cp  - stub_start + 0xffffe000)
        idt_entry_empty
        idt_entry_empty
        idt_entry_empty
        idt_entry_empty
        idt_entry_empty
        idt_entry_empty
        idt_entry (stub_hv  - stub_start + 0xffffe000)
        idt_entry (stub_vc  - stub_start + 0xffffe000)
        idt_entry (stub_sx  - stub_start + 0xffffe000)
idt_end:

idt_desc:
        .word   idt_end - idt_start - 1
        .quad   idt_start

        .section .text
.global idt_load
idt_load:
        lidt    idt_desc(%rip)
        retq
