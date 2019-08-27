#![feature(asm, global_asm)]
global_asm! { r#"
    .section    .data, ""
    .p2align    4
low_nibble_mask:
    .quad 0x0000000000000010
    .quad 0x00000C01040A0800
    .quad 0x0000000000000010
    .quad 0x00000C01040A0800
high_nibble_mask:
    .quad 0x0400040002110008
    .quad 0x0000000000000000
    .quad 0x0400040002110008
    .quad 0x0000000000000000
structural_mask:
    .zero 32, 0x07
whitespace_mask:
    .zero 32, 0x18
no_highest_bit_in_byte:
    .zero 32, 0x7F
"# }

fn main() {
    let src = br#"12      ::: \\\\ ,,,,, {{{{{{ {}{ 123123123 }{} }}}}}} 456456456"#;
    let (w, s) = process(src); 
    for _ in 0..200_000_000 {
        let (_w, _s) = process(src); 
    }
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
        vpxor ymm0, ymm0, ymm0

        vmovdqu ymm1, [rdi]
        vmovdqa ymm2, [rip + low_nibble_mask] 
        vpshufb ymm2, ymm2, ymm1
        vpsrlq ymm1, ymm1, 4 
        vpand ymm1, ymm1, [rip + no_highest_bit_in_byte] 
        vmovdqa ymm3, [rip + high_nibble_mask] 
        vpshufb ymm3, ymm3, ymm1
        vpand ymm2, ymm2, ymm3 
        vmovdqa ymm3, [rip + structural_mask] 
        vpand ymm3, ymm3, ymm2 
        vpcmpeqb ymm3, ymm3, ymm0
        vpmovmskb rdx, ymm3
        vmovdqa ymm3, [rip + whitespace_mask]
        vpand ymm3, ymm3, ymm2
        vpcmpeqb ymm3, ymm3, ymm0
        vpmovmskb rax, ymm3

        vmovdqu ymm1, [rdi + 20h]
        vmovdqa ymm2, [rip + low_nibble_mask] 
        vpshufb ymm2, ymm2, ymm1
        vpsrlq ymm1, ymm1, 4 
        vpand ymm1, ymm1, [rip + no_highest_bit_in_byte] 
        vmovdqa ymm3, [rip + high_nibble_mask] 
        vpshufb ymm3, ymm3, ymm1
        vpand ymm2, ymm2, ymm3 
        vmovdqa ymm3, [rip + structural_mask] 
        vpand ymm3, ymm3, ymm2 
        vpcmpeqb ymm3, ymm3, ymm0
        vpmovmskb r8, ymm3
        shl r8, 20h
        or rdx, r8
        vmovdqa ymm3, [rip + whitespace_mask]
        vpand ymm3, ymm3, ymm2
        vpcmpeqb ymm3, ymm3, ymm0
        vpmovmskb r8, ymm3
        shl r8, 20h
        or rax, r8
    ":"={rax}"(whitespace), "={rdx}"(structurals)
    :"{rdi}"(src.as_ptr())
    :"ymm0","ymm1","ymm2","ymm3","r8"
    :"intel") };
    (whitespace, structurals)
}
