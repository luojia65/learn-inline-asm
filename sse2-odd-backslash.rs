#![feature(asm, global_asm)]
global_asm! { r#"
    .section	.data, ""
sixteen_backslashes:
    .long 0x5C5C5C5C
    .long 0x5C5C5C5C
    .long 0x5C5C5C5C
    .long 0x5C5C5C5C
even_bits_64:
    .long 0x55555555
    .long 0x55555555
    .long 0x55555555
    .long 0x55555555
"# }

fn main() {
    let src = br#" important \\\\\ i\portant \\\\f important \\\ff important \\uff"#;
    let ans = process(src);  
    println!("{:064b}", ans);
}

fn process(src: &[u8]) -> u64 {
    let ans: u64;
    unsafe { asm!("
        movdqu xmm1, [rip + sixteen_backslashes]
        movdqu xmm0, [r8]
        pcmpeqb xmm0, xmm1
        pmovmskb r9, xmm0
        mov r10, r9
        movdqu xmm0, [r8+10h]
        pcmpeqb xmm0, xmm1
        pmovmskb r9, xmm0
        shl r9, 10h
        or r10, r9
        movdqu xmm0, [r8+20h]
        pcmpeqb xmm0, xmm1
        pmovmskb r9, xmm0
        shl r9, 20h
        or r10, r9
        movdqu xmm0, [r8+30h]
        pcmpeqb xmm0, xmm1
        pmovmskb r9, xmm0
        shl r9, 30h
        or r10, r9          # r10: bs_bits
        mov r8, r10
        shl r8, 1
        not r8
        and r8, r10         # r8: start_edges
        mov r9, [rip + even_bits_64]    # r9: even_start_mask
        mov rax, r8
        and rax, r9         # rax: even_starts
        not r9
        and r9, r8          # r9: odd_starts
        add rax, r10        # rax: even_carries
        add r9, r10         # r9: odd_carries
        not r10             # r10: !bs_bits
        and rax, r10        # rax: even_carry_ends
        and r9, r10         # r9: odd_carry_ends
        mov r10, [rip + even_bits_64] # r10: even_bits
        and r9, r10         # r9: odd_start_even_end
        not r10
        and rax, r10        # rax: even_start_odd_end
        or rax, r9
    ":"={rax}"(ans)
    :"{r8}"(src.as_ptr())
    :"xmm0", "xmm1", "rax", "r8", "r9", "r10"
    :"intel") };
    ans
}

