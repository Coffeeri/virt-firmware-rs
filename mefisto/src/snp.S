        .code32
        .section .text32, "ax"
        .global init_snp

init_snp:
        mov	$(1 << (51-32)), %ebx	// FIXME: c-bit 51 hardcoded
        mov	$pg2, %eax		// level 2 page tables
        mov	$4 * 512, %ecx		// 4 page tables, 512 entries each

pteloop:
        add	$4, %eax
        mov	%ebx, (%eax)
        add	$4, %eax
        loop	pteloop
        ret
