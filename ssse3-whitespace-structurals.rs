#![feature(asm, global_asm)]
global_asm! { r#"
    .section    .data, ""
    .p2align    4
low_nibble_mask:
    .quad 0xF000000000000000
    .quad 0x00080A04010C0000
high_nibble_mask:
    .quad 0x0800110200040004
    .quad 0x0000000000000000
structural_mask:
    .zero 16, 0x07
whitespace_mask:
    .zero 16, 0x18
no_highest_bit_in_byte:
    .zero 16, 0x7F
"# }

fn main() {
    let src = br#" important \\\\\ i\portant \\\\f important \\\ff important \\uff"#;
    let (w, s) = process(src); 
    println!("{:064b}", w);
    println!("{:064b}", s);
}

#[inline(never)]
fn process(src: &[u8]) -> (u64, u64) {
    let whitespace: u64;
    let structurals: u64;
    unsafe { asm!("
        lea rax, [rip + no_highest_bit_in_byte]
        movdqa xmm0, [rax] # xmm0 0x7f
        movdqu xmm1, [rdi] # xmm1 input
        lea rax, [rip + low_nibble_mask]
        movdqa xmm2, [rax] # xmm2 lo_msk
        pshufb xmm2, xmm1 # xmm2 and_1
        psrlq xmm1, 4 # xmm1 input >> 4
        pand xmm1, xmm0 # xmm1 hi_input
        lea rax, [rip + high_nibble_mask]
        movdqa xmm3, [rax] # xmm3 hi_msk
        pshufb xmm3, xmm1 # xmm3 and_2
        pand xmm2, xmm3 # xmm2 v_V0
        pxor xmm3, xmm3 # xmm3 0
        lea rax, [rip + structural_mask]
        movdqa xmm1, [rax] # xmm1 struct_msk
        pand xmm1, xmm2 # xmm1 cmpeq_in1
        pcmpeqb xmm1, xmm3 # xmm1 struct
        pmovmskb rdx, xmm1
        lea rax, [rip + whitespace_mask]
        movdqa xmm1, [rax] # xmm1 white_msk
        pand xmm1, xmm2 
        pcmpeqb xmm2, xmm3
        pmovmskb rax, xmm1
    ":"={rax}"(whitespace), "={rdx}"(structurals)
    :"{rdi}"(src.as_ptr())
    :
    :"intel") };
    (whitespace, structurals)
}

