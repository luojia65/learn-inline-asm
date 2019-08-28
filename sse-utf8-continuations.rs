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
sixteen_unicode_max:
    .zero 16, 0xF4
"# }

fn main() {
    let src = b"\xf4\xd0\xe0\xf000  ::: 1234 ,,,,, {{{{{{ {}{ 123123123 }{} }}}}}} 456456456";
    let _ = process(src); 
}

#[inline(never)]
fn process(src: &[u8]) -> () {
    let outh: u64; let outl: u64;
    unsafe { asm!("
        mov rcx, 0
        pxor xmm0, xmm0 # xmm0: prev_carries
        pxor xmm1, xmm1 # xmm1: has_error 
process_loop:
        movdqu xmm2, [rdi + rcx] # xmm2: current_bytes
        movdqa xmm3, xmm2
    # check unicode max 0xf4 into has_error
        psubusb xmm3, [rip + sixteen_unicode_max]
        por xmm1, xmm3
    # get continuation lengths
        movdqa xmm3, xmm2
        psrlq xmm3, 4
        pand xmm3, [rip + lo_nibble_filter] # xmm3: hi nibble now 
        movdqa xmm4, [rip + continuation_length]
        pshufb xmm4, xmm3 # xmm4: initial lengths
    # get carried continuations
        movdqa xmm5, xmm4
        palignr xmm5, xmm0, 15 
        psubusb xmm5, [rip + sixteen_u8_ones] # xmm5: right1
        paddb xmm5, xmm4; # xmm5: sum
        movdqa xmm6, xmm5; # xmm6: sum
        palignr xmm6, xmm0, 14 
        psubusb xmm6, [rip + sixteen_u8_twos] # xmm6: right2
        paddb xmm5, xmm6; # xmm5: carried_continuations 
    # check continuations



        // add rcx, 64
        // cmp rcx, rsi
        // jb process_loop

        movq rax, xmm5
        pextrq rdx, xmm5, 1
    ":"={rax}"(outh),"={rdx}"(outl)
    :"{rdi}"(src.as_ptr()), "{rsi}"(src.len())
    :"xmm0","rcx" // todo
    :"intel") };
    println!("{:016X}{:016X}", outl, outh);
}

