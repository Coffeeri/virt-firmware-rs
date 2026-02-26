        .code32
        .section .text32, "ax"
        .global init_snp

init_snp:
        mov	$0xc0010130, %ecx	// MSR_SEV_GHCB
        mov	$0x002, %eax		// SEV INFO request
        xor	%edx, %edx
        wrmsr
        vmgexit
        rdmsr
        shr	$24, %eax		// cbit position at [31:24]

        sub	$32, %eax		// %ebx = (1 << (cbit - 32))
        mov	%al, %cl
        mov	$1, %ebx
        shl	%cl, %ebx

        mov	$pg2, %eax		// level 2 page tables
        mov	$4 * 512, %ecx		// 4 page tables, 512 entries each

pteloop:
        add	$4, %eax		// skip low 32 bits of pte
        mov	%ebx, (%eax)		// write high 32 bits of pte
        add	$4, %eax
        loop	pteloop
        ret
