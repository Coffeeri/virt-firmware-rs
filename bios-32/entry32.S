        .code32
        .section .text

startram:
        // setup stack @ 640k
        mov	$stack_end, %esp

        // jump to C code
        mov	%ebp, %eax
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

        // jump to ram
        mov	$startram, %eax
        jmp	*%eax

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
