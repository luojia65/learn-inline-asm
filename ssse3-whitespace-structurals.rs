#![feature(asm, global_asm)]
global_asm! { r#"
    .section    .data, ""
    .p2align    4
low_nibble_mask:
    .quad 0x0000000000000010
    .quad 0x00000C01040A0800
high_nibble_mask:
    .quad 0x0400040002110008
    .quad 0x0000000000000000
structural_mask:
    .zero 16, 0x07
whitespace_mask:
    .zero 16, 0x18
no_highest_bit_in_byte:
    .zero 16, 0x7F
"# }

fn main() {
    let src = br#"12      ::: \\\\ ,,,,, {{{{{{ {}{ 123123123 }{} }}}}}} 456456456"#;
    let (w, s) = process(src); 
    println!("{:064b}", w);
    println!("{:064b}", s);
    for b in src.iter().rev() {
        print!("{}", String::from_utf8_lossy(&[*b]));
    } 
    println!()
}

#[inline(never)]
fn process(src: &[u8]) -> (u64, u64) {
    let whitespace: u64;
    let structurals: u64;
    unsafe { asm!("
        movdqu xmm0, [rdi] # xmm0 input
        movdqa xmm1, [rip + low_nibble_mask] # xmm1 lo_msk
        pshufb xmm1, xmm0 # xmm1 and_1
        psrlq xmm0, 4 # xmm0 input >> 4
        pand xmm0, [rip + no_highest_bit_in_byte]  # xmm0 hi_input
        movdqa xmm2, [rip + high_nibble_mask] # xmm1 hi_msk
        pshufb xmm2, xmm0 # xmm2 and_2
        pand xmm1, xmm2 # xmm1 v_V0
        pxor xmm0, xmm0 # xmm0 0
        movdqa xmm2, [rip + structural_mask] # xmm2 struct_msk
        pand xmm2, xmm1 # xmm2 cmpeq_in1
        pcmpeqb xmm2, xmm0 # xmm2 struct
        pmovmskb rdx, xmm2
        movdqa xmm2, [rip + whitespace_mask] # xmm2 white_msk
        pand xmm2, xmm1
        pcmpeqb xmm2, xmm0
        pmovmskb rax, xmm2
    ":"={rax}"(whitespace), "={rdx}"(structurals)
    :"{rdi}"(src.as_ptr())
    :
    :"intel") };
    (whitespace, structurals)
}

