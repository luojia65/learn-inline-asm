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
    let src = b"\xf4\x80\x80\x8000  ::: 1234 ,,,,, {{{{{{ {}{ 123123123 }{} }}}}}} 456456456";
    let _ = process(src); 
}

#[inline(never)]
fn process(src: &[u8]) -> () {
    let outh: u64; let outl: u64;
    unsafe { asm!("
        pxor xmm0, xmm0 # xmm0: prev_carried_continuations
        pxor xmm1, xmm1 # xmm1: has_error 
        pxor xmm2, xmm2 # xmm2: prev_high_nibbles
        mov rcx, 0
process_loop:
        movdqu xmm3, [rdi + rcx] # xmm3: current_bytes
        movdqa xmm4, xmm3
    # check unicode max 0xf4 into has_error
        psubusb xmm4, [rip + sixteen_unicode_max]
        por xmm1, xmm4
    # get continuation lengths
        movdqa xmm4, xmm3
        psrlq xmm4, 4
        pand xmm4, [rip + lo_nibble_filter] # xmm4: hi nibble cur 
        movdqa xmm5, [rip + continuation_length]
        pshufb xmm5, xmm4 # xmm5: initial lengths
    # get carried continuations
        movdqa xmm6, xmm5
        palignr xmm6, xmm0, 15 
        psubusb xmm6, [rip + sixteen_u8_ones] # xmm6: right1
        paddb xmm6, xmm5 # xmm6: sum
        movdqa xmm7, xmm6 # xmm7: sum
        palignr xmm7, xmm0, 14 
        psubusb xmm7, [rip + sixteen_u8_twos] # xmm7: right2
        paddb xmm6, xmm7 # xmm6: carried_continuations 
        movdqa xmm0, xmm6 # xmm0: carried_continuations
    # check continuations
        pxor xmm7, xmm7 # xmm7: 0
        pcmpgtb xmm6, xmm5 # xmm6: eq_1
        pcmpgtb xmm5, xmm7 # xmm5: eq_2
        pcmpeqb xmm5, xmm6 # xmm5: overunder
        por xmm1, xmm5 
    # get offset current bytes
        palignr xmm6, xmm0, 15 
        // ???
    # move current high nibbles to prev high nibbles
        movdqa xmm2, xmm4
        add rcx, 16
        cmp rcx, rsi
        jb process_loop

        movq rax, xmm1
        pextrq rdx, xmm1, 1
    ":"={rax}"(outh),"={rdx}"(outl)
    :"{rdi}"(src.as_ptr()), "{rsi}"(src.len())
    :"xmm0","rcx" // todo
    :"intel") };
    println!("{:016X}{:016X}", outl, outh);
}

