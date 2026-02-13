        .code64
        .section .text

start64:
        // setup stack
        mov	$stack_end, %rsp

        // jump to C/rust code
        xor	%rdi, %rdi
        mov	%ebp, %edi
        call	main

halt:
        hlt
        jmp     halt

        .code32
        .section .text32, "ax"

start32x:
        mov	$32, %ebp	// startup via 32-bit mode

        // load our gdt
        lgdtl	0xffffff70
        ljmpl	$8, $start32

start32:
        // load data segments
        mov	$0x10, %ax
        mov	%ax, %ds
        mov	%ax, %es
        mov	%ax, %ss

        // setup stack
        mov	$stack_end, %esp

        // copy 64-bit code + data to ram
        mov     $copy_from, %esi
        mov     $copy_start, %edi
        mov     $copy_end, %ecx
        sub     $copy_start, %ecx
        rep     movsb

        // setup paging
        mov	$pg4, %eax
        mov	%eax, %cr3

        mov	%cr4, %eax
        bts	$4, %eax // PSE
        bts	$5, %eax // PAE
        mov	%eax, %cr4

        // enable long mode
        mov     $0xc0000080, %ecx
        rdmsr
        bts     $8, %eax  // LME
        bts     $11, %eax // NXE
        wrmsr

        // enable paging
        mov	%cr0, %eax
        bts	$16, %eax // WP
        bts	$31, %eax // PG
        mov	%eax, %cr0

        // enter long mode
        ljmpl	$0x18, $start64

        .section .pgtable, "aw"
        .align 4096
pg4:
        // level 4 page directory
        .long   pg3 + 0x23 // present, rw, accessed
        .long   0

        .align 4096
pg3:
        // level 3 page directory
        .set    pgt, pg2
        .rept   4
        .long   pgt + 0x23 // present, rw, accessed
        .long   0
        .set    pgt, pgt + (1 << 12)
        .endr

        .align  4096
pg2:
        // level 2 page tables, using 2M pages
        .set    addr, 0
        .rept   512 * 4
        .long   addr + 0xe7 // present, rw, user, accessed, dirty, largepage
        .long   0
        .set    addr, addr + (1 << 21)
        .endr

        .align 4096
pge:

        .section .gdt, "a"
gdts:
        // f000:ff00 / ffffff00
        // null descriptor
        .quad   0
        // flat 32-bit code segment
        .word   0xffff
        .word   0
        .byte   0
        .byte   0x9b
        .byte   0xcf
        .byte   0
        // flat 32-bit data segment
        .word   0xffff
        .word   0
        .byte   0
        .byte   0x93
        .byte   0xcf
        .byte   0
        // flat 64-bit code segment
        .word   0xffff
        .word   0
        .byte   0
        .byte   0x9b
        .byte   0xaf
        .byte   0
gdte:

        .section .gdtd, "a"
gdtd:
        // f000:ff70 / ffffff70
        // gdt descriptor
        .word   gdte - gdts - 1
        .long   0xffffff00 // gdts

        .code16
        .section .text16, "ax"

start16:
        // f000:ff80 / ffffff80
        lgdtl	%cs:0xff70
        mov	$1, %eax	// protected mode
        mov	%eax, %cr0
        mov	$16, %ebp	// startup via 16-bit mode
        ljmpl	$8, $start32

        .section .entry16, "ax"
        .global entry16
entry16:
        // f000:fff0 / fffffff0
        mov     %cr0, %eax
        test    $1,%al
        jz      is16
        .code32
        jmp	start32x
        .code16
is16:
        jmp     start16
entry16e:
        .fill 16 - (entry16e - entry16),1,0
