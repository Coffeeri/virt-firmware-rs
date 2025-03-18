        .code64
        .section .text

start64:
        // setup stack @ 640k
        mov	$0xa0000, %rsp

        // jump to C code
        call	main

halt:
        hlt
        jmp     halt

        .code32
        .section .text32, "ax"

start32:
        // load data segments
        mov	$0x10, %ax
        mov	%ax, %ds
        mov	%ax, %es
        mov	%ax, %ss

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
        bts     $8, %eax // LME
        wrmsr

        // enable paging
        mov	%cr0, %eax
        bts	$31, %eax // PG
        mov	%eax, %cr0

        // enter long mode
        ljmpl	$0x18, $start64

        .section .pgtable, "a"
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
pg3:
        // level 3 page directory
        .set    pgt, pg2
        .rept   4
        .long   pgt + 0x23 // present, rw, accessed
        .long   0
        .set    pgt, pgt + (1 << 12)
        .endr

        .align 4096
pg4:
        // level 4 page directory
        .long   pg3 + 0x23 // present, rw, accessed
        .long   0

        .align 4096
pge:

        .section .gdt, "a"
gdts:
        // f000:ff00
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
        // f000:ff70
        // gdt descriptor
        .word   gdte - gdts - 1
        .long   0x000fff00 // gdts

        .code16
        .section .text16, "ax"

start16:
        // f000:ff80
        lgdtw	%cs:0xff70
        mov	$1, %eax	// protected mode
        mov	%eax, %cr0
        ljmpl	$8, $start32

        .section .entry16, "ax"
        .global	entry16
entry16:
        // f000:fff0
        jmp	start16
entry16e:
        .fill 16 - (entry16e - entry16),1,0
