#![feature(asm, global_asm)]
global_asm! { r#"
    .section    .data, ""
    .p2align    4
continuation_length:
    .quad 0x0101010101010101 
    .quad 0x0403020200000000
lo_nibble_filter:
    .zero 16, 0x0F
sixteen_u8_ones:
    .zero 16, 0x01
sixteen_u8_twos:
    .zero 16, 0x02
"# }

fn main() {
    let src = b"\xc0\xd0\xe0\xf000  ::: 1234 ,,,,, {{{{{{ {}{ 123123123 }{} }}}}}} 456456456";
    let _ = process(src); 
}

#[inline(never)]
fn process(src: &[u8]) -> () {
    let outh: u64; let outl: u64;
    unsafe { asm!("
        pxor xmm0, xmm0 // xmm0: prev
        // movdqa xmm0, [rip + continuation_length]
        // movdqu xmm1, [rdi]
        // psrlq xmm1, 4
        // pand xmm1, [rip + lo_nibble_filter]
        // pshufb xmm0, xmm1
        // palignr xmm0, xmm1, 15
        // movq rax, xmm0
        // pextrq rdx, xmm0, 1
    ":"={rax}"(outh),"={rdx}"(outl)
    :"{rdi}"(src.as_ptr())
    :"xmm0"
    :"intel") };
    println!("{:016X}{:016X}", outl, outh);
}

