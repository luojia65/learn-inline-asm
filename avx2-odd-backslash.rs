#![feature(asm, global_asm)]
global_asm! { r#"
    .section	.data, ""
    .p2align    4
all_backslashes:
    .zero 32, 0x5C
"# }

fn main() {
    let src = br#" important \\\\\ i\portant \\\\f important \\\ff important \\uff"#;
    let ans = process(src); 
    println!("{:064b}", ans);
}

fn process(src: &[u8]) -> u64 {
    let ans: u64;
    unsafe { asm!("
        vzeroall
    # ymm1: input
    # ymm2: tmp
        vlddqu ymm1, [rdi];
        vpcmpeqb ymm2, ymm1, [rip + all_backslashes]
        vpmovmskb r10, ymm2
        vlddqu ymm1, [rdi + 32]
        vpcmpeqb ymm2, ymm1, [rip + all_backslashes]
        vpmovmskb r8, ymm2
        shl r8, 32
        or r10, r8
        mov r8, r10
        shl r8, 1
        not r8
        and r8, r10         # r8: start_edges
        mov r9, 0x5555555555555555    # r9: even_start_mask
        mov rax, r8
        and rax, r9         # rax: even_starts
        not r9
        and r9, r8          # r9: odd_starts
        add rax, r10        # rax: even_carries
        add r9, r10         # r9: odd_carries
        not r10             # r10: !bs_bits
        and rax, r10        # rax: even_carry_ends
        and r9, r10         # r9: odd_carry_ends
        mov r10, 0x5555555555555555 # r10: even_bits
        and r9, r10         # r9: odd_start_even_end
        not r10
        and rax, r10        # rax: even_start_odd_end
        or rax, r9
    ":"={rax}"(ans)
    :"{rdi}"(src.as_ptr())
    :"ymm1","ymm2","r8","r9","r10"
    :"intel") };
    ans
}

