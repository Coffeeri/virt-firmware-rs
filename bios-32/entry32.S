        .code32
        .section .text

start32:
        // load data segments
        mov	$0x10, %ax
        mov	%ax, %ds
        mov	%ax, %es
        mov	%ax, %ss

        // setup stack @ 640k
        mov	$0xa0000, %esp

        // jump to C code
        call	main

halt:
        hlt
        jmp     halt

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
        ljmpl	$8, $start32

        .section .entry16, "ax"
        .global entry16
entry16:
        // f000:fff0
        jmp	start16
entry16e:
        .fill 16 - (entry16e - entry16),1,0
