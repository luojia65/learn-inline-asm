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
    for i in 0..200_000_000 {
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
        pxor xmm0, xmm0 

        movdqu xmm1, [rdi]
        movdqa xmm2, [rip + low_nibble_mask] 
        pshufb xmm2, xmm1
        psrlq xmm1, 4 
        pand xmm1, [rip + no_highest_bit_in_byte] 
        movdqa xmm3, [rip + high_nibble_mask] 
        pshufb xmm3, xmm1
        pand xmm2, xmm3 
        movdqa xmm3, [rip + structural_mask] 
        pand xmm3, xmm2 
        pcmpeqb xmm3, xmm0
        pmovmskb rdx, xmm3
        movdqa xmm3, [rip + whitespace_mask]
        pand xmm3, xmm2
        pcmpeqb xmm3, xmm0
        pmovmskb rax, xmm3
        
        movdqu xmm1, [rdi + 10h]
        movdqa xmm2, [rip + low_nibble_mask] 
        pshufb xmm2, xmm1
        psrlq xmm1, 4 
        pand xmm1, [rip + no_highest_bit_in_byte] 
        movdqa xmm3, [rip + high_nibble_mask] 
        pshufb xmm3, xmm1
        pand xmm2, xmm3 
        movdqa xmm3, [rip + structural_mask] 
        pand xmm3, xmm2 
        pcmpeqb xmm3, xmm0
        pmovmskb r8, xmm3
        shl r8, 10h
        or rdx, r8
        movdqa xmm3, [rip + whitespace_mask]
        pand xmm3, xmm2
        pcmpeqb xmm3, xmm0
        pmovmskb r8, xmm3
        shl r8, 10h
        or rax, r8

        movdqu xmm1, [rdi + 20h]
        movdqa xmm2, [rip + low_nibble_mask] 
        pshufb xmm2, xmm1
        psrlq xmm1, 4 
        pand xmm1, [rip + no_highest_bit_in_byte] 
        movdqa xmm3, [rip + high_nibble_mask] 
        pshufb xmm3, xmm1
        pand xmm2, xmm3 
        movdqa xmm3, [rip + structural_mask] 
        pand xmm3, xmm2 
        pcmpeqb xmm3, xmm0
        pmovmskb r8, xmm3
        shl r8, 20h
        or rdx, r8
        movdqa xmm3, [rip + whitespace_mask]
        pand xmm3, xmm2
        pcmpeqb xmm3, xmm0
        pmovmskb r8, xmm3
        shl r8, 20h
        or rax, r8

        movdqu xmm1, [rdi + 30h]
        movdqa xmm2, [rip + low_nibble_mask] 
        pshufb xmm2, xmm1
        psrlq xmm1, 4 
        pand xmm1, [rip + no_highest_bit_in_byte] 
        movdqa xmm3, [rip + high_nibble_mask] 
        pshufb xmm3, xmm1
        pand xmm2, xmm3 
        movdqa xmm3, [rip + structural_mask] 
        pand xmm3, xmm2 
        pcmpeqb xmm3, xmm0
        pmovmskb r8, xmm3
        shl r8, 30h
        or rdx, r8
        movdqa xmm3, [rip + whitespace_mask]
        pand xmm3, xmm2
        pcmpeqb xmm3, xmm0
        pmovmskb r8, xmm3
        shl r8, 30h
        or rax, r8
    ":"={rax}"(whitespace), "={rdx}"(structurals)
    :"{rdi}"(src.as_ptr())
    :"xmm0","xmm1","xmm2","xmm3","r8"
    :"intel") };
    (whitespace, structurals)
}

