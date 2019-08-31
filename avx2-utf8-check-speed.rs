#![feature(asm, global_asm)]
global_asm! { r#"
    .section    .data, ""
    .p2align    4
continuation_length:
    .quad 0x0101010101010101 
    .quad 0x0403020200000000
    .quad 0x0101010101010101 
    .quad 0x0403020200000000
lo_nibble_filter:
    .zero 32, 0x0F
all_u8_ones:
    .zero 32, 0x01
all_u8_twos:
    .zero 32, 0x02
all_ed_bytes:
    .zero 32, 0xED
all_f4_bytes:
    .zero 32, 0xF4
all_9f_bytes:
    .zero 32, 0x9F
all_8f_bytes:
    .zero 32, 0x8F
initial_min_mask:
    .quad 0x8080808080808080
    .quad 0xF1E180C280808080
    .quad 0x8080808080808080
    .quad 0xF1E180C280808080
second_min_mask:
    .quad 0x8080808080808080
    .quad 0x90A07F7F80808080
    .quad 0x8080808080808080
    .quad 0x90A07F7F80808080
"# }

// This program does not check for illegal 0xC0 and 0xC1 chars
fn main() {
    let mut src: Vec<u8> = Vec::new();
    for _ in 1..=1024*1024*1024 {
        src.push(0x01);
    }
    println!("Size: {} bytes", src.len());
    use std::time::Instant;
    let a = Instant::now();
    let c = process(&src); 
    let b = Instant::now();
    let dur = b - a;
    println!("Result: {}", c);
    println!("Time: {} ns", dur.as_nanos());
    let speed = (src.len() as f64)/(dur.as_nanos() as f64 / 1_000_000_000.0)/(1024.0*1024.0*1024.0);
    println!("Speed: {:.06} GiB/s", speed);
}

#[inline(never)]
fn process(src: &[u8]) -> bool {
    let out: bool;
    unsafe { asm!("
    # ymm0: has_error
    # ymm1: prev_raw, off1_current_bytes
    # ymm2: prev_high_nibbles, off1_high_nibble
    # ymm3: prev_carried_continuations, cur_carried_continuations
    # ymm4: cur_raw
    # ymm5: cur_high_nibbles
    # ymm6: initial_length
    # ymm7: tmp
    # ymm8: tmp
    # ymm9: zero
        vzeroall
        mov rcx, 0
process_loop:
    # load cur_raw 
        vlddqu ymm4, [rdi + rcx]
    # check unicode max 0xf4 into has_error
        vpsubusb ymm5, ymm4, [rip + all_f4_bytes]
        vpor ymm0, ymm5, ymm0
    # get current continuation lengths
        vpsrlq ymm5, ymm4, 4
        vpand ymm5, ymm5, [rip + lo_nibble_filter]
        vmovdqa ymm6, [rip + continuation_length]
        vpshufb ymm6, ymm6, ymm5
    # get current carried continuations
        vperm2i128 ymm8, ymm3, ymm6, 0x21
        vpalignr ymm8, ymm6, ymm8, 15
        vpsubusb ymm8, ymm8, [rip + all_u8_ones]
        vpaddb ymm8, ymm8, ymm6
        vperm2i128 ymm7, ymm3, ymm8, 0x21
        vpalignr ymm7, ymm8, ymm7, 14
        vpsubusb ymm7, ymm7, [rip + all_u8_twos]
        vpaddb ymm3, ymm8, ymm7
    # check continuations
        vpcmpgtb ymm8, ymm3, ymm6
        vpcmpgtb ymm7, ymm6, ymm9 
        vpcmpeqb ymm8, ymm8, ymm7 
        vpor ymm0, ymm0, ymm8
    # get offset current bytes
        vperm2i128 ymm1, ymm1, ymm4, 0x21
        vpalignr ymm1, ymm4, ymm1, 15
    # check first continuation max
        vpcmpeqb ymm7, ymm1, [rip + all_ed_bytes]
        vpcmpgtb ymm8, ymm4, [rip + all_9f_bytes]
        vpand ymm7, ymm7, ymm8
        vpor ymm0, ymm0, ymm7
        vpcmpeqb ymm7, ymm1, [rip + all_f4_bytes]
        vpcmpgtb ymm8, ymm4, [rip + all_8f_bytes]
        vpand ymm7, ymm7, ymm8
        vpor ymm0, ymm0, ymm7
    # check overlong
        vperm2i128 ymm2, ymm2, ymm5, 0x21
        vpalignr ymm2, ymm5, ymm2, 15
        vmovdqa ymm7, [rip + initial_min_mask]
        vpshufb ymm7, ymm7, ymm2
        vpcmpgtb ymm8, ymm7, ymm1
        vmovdqa ymm7, [rip + second_min_mask]
        vpshufb ymm7, ymm7, ymm2
        vpcmpgtb ymm7, ymm7, ymm4
        vpand ymm7, ymm7, ymm8
        vpor ymm0, ymm0, ymm7
    # move current to prev
        vmovdqa ymm1, ymm4
        vmovdqa ymm2, ymm5
        add rcx, 32
        cmp rcx, rsi
        jb process_loop
    # test result
        vptest ymm0, ymm0
        jz result_no_error
        mov rax, 0
        jmp result_has_error
result_no_error:
        mov rax, 1
result_has_error:
    ":"={rax}"(out)
    :"{rdi}"(src.as_ptr()), "{rsi}"(src.len())
    :"ymm0","ymm1","ymm2","ymm3","ymm4","ymm5","ymm6","ymm7","ymm8","ymm9","rcx"
    :"intel") };
    out
}
