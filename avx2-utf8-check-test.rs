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

fn main() {
    let mut src = Vec::new();
    // U+0000..U+007F     00..7F
    for i in 0..=0x7F {
        src.push(i)
    }
    // U+0080..U+07FF     C2..DF   80..BF
    for i in 0xC2..=0xDF {
        for j in 0x80..=0xBF {
            src.push(i);
            src.push(j);
        }
    }
    // U+0800..U+0FFF     E0       A0..BF   80..BF
    for i in 0xA0..=0xBF {
        for j in 0x80..=0xBF {
            src.push(0xE0);
            src.push(i);
            src.push(j);
        }
    }
    // U+1000..U+CFFF     E1..EC   80..BF   80..BF
    for i in 0xE1..=0xEC {
        for j in 0x80..=0xBF {
            for k in 0x80..=0xBF {
                src.push(i);
                src.push(j);
                src.push(k);
            }
        }
    }
    // U+D000..U+D7FF     ED       80..9F   80..BF
    // U+E000..U+FFFF     EE..EF   80..BF   80..BF
    // U+10000..U+3FFFF   F0       90..BF   80..BF   80..BF
    // U+40000..U+FFFFF   F1..F3   80..BF   80..BF   80..BF
    // U+100000..U+10FFFF F4       80..8F   80..BF   80..BF
    for i in 0x80..=0x8F {
        for j in 0x80..=0xBF {
            for k in 0x80..=0xBF {
                src.push(0xF4);
                src.push(i);
                src.push(j);
                src.push(k);
            }
        }
    }
    println!("Length: {}", src.len());
    let _ = process(&src); 
    // println!("Is string a valid UTF-8? {}", out);
}


#[inline(never)]
fn process(src: &[u8]) -> () {
    let out = [0u64; 4];
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
    // # get offset current bytes
    //     vpalignr ymm1, ymm4, ymm1, 15
    // # check first continuation max
    //     vpcmpeqb ymm7, ymm1, [rip + all_ed_bytes]
    //     vpcmpgtb ymm8, ymm4, [rip + all_9f_bytes]
    //     vpand ymm7, ymm7, ymm8
    //     vpor ymm0, ymm0, ymm7
    //     vpcmpeqb ymm7, ymm1, [rip + all_f4_bytes]
    //     vpcmpgtb ymm8, ymm4, [rip + all_8f_bytes]
    //     vpand ymm7, ymm7, ymm8
    //     vpor ymm0, ymm0, ymm7
    // # check overlong
    //     vpalignr ymm2, ymm5, ymm2, 15
    //     vmovdqa ymm7, [rip + initial_min_mask]
    //     vpshufb ymm7, ymm7, ymm2
    //     vpcmpgtb ymm8, ymm7, ymm1
    //     vmovdqa ymm7, [rip + second_min_mask]
    //     vpshufb ymm7, ymm7, ymm2
    //     vpcmpgtb ymm7, ymm7, ymm4
    //     vpand ymm7, ymm7, ymm8
    //     vpor ymm0, ymm0, ymm7
    // # move current to prev
    //     vmovdqa ymm1, ymm4
    //     vmovdqa ymm2, ymm5

        // add rcx, 64
        // cmp rcx, rsi
        // jb process_loop
        
        vmovdqu [rax], ymm0
    ":
    :"{rdi}"(src.as_ptr()), "{rsi}"(src.len()), "{rax}"(out.as_ptr())
    :"ymm0","ymm1","ymm2","ymm3","ymm4","ymm5","ymm6","ymm7","ymm8","ymm9","rcx"
    :"intel") };
    println!("{:016X}{:016X}{:016X}{:016X}", out[3], out[2], out[1], out[0]);
}

// #[inline(never)]
// fn process(src: &[u8]) -> bool {
//     let out: bool;
//     unsafe { asm!("
//         vzeroall
//         mov rcx, 0
// process_loop:
//         vlddqu ymm4, [rdi + rcx]
//         vpsubusb ymm5, ymm4, [rip + all_f4_bytes]
//         vpor ymm0, ymm5, ymm0
//         vpsrlq ymm5, ymm4, 4
//         vpand ymm5, ymm5, [rip + lo_nibble_filter]
//         vmovdqa ymm6, [rip + continuation_length]
//         vpshufb ymm6, ymm6, ymm5
        // vperm2i128 ymm8, ymm3, ymm6, 0x21
        // vpalignr ymm8, ymm6, ymm8, 15
        // vpsubusb ymm8, ymm8, [rip + all_u8_ones]
        // vpaddb ymm8, ymm8, ymm6
        // vperm2i128 ymm7, ymm3, ymm8, 0x21
        // vpalignr ymm7, ymm8, ymm7, 14
        // vpsubusb ymm7, ymm7, [rip + all_u8_twos]
        // vpaddb ymm3, ymm8, ymm7
//         vpcmpgtb ymm8, ymm3, ymm6
//         vpcmpgtb ymm7, ymm6, ymm9 
//         vpcmpeqb ymm8, ymm8, ymm7 
//         vpor ymm0, ymm0, ymm8
//         vpalignr ymm1, ymm4, ymm1, 15
//         vpcmpeqb ymm7, ymm1, [rip + all_ed_bytes]
//         vpcmpgtb ymm8, ymm4, [rip + all_9f_bytes]
//         vpand ymm7, ymm7, ymm8
//         vpor ymm0, ymm0, ymm7
//         vpcmpeqb ymm7, ymm1, [rip + all_f4_bytes]
//         vpcmpgtb ymm8, ymm4, [rip + all_8f_bytes]
//         vpand ymm7, ymm7, ymm8
//         vpor ymm0, ymm0, ymm7
//         vpalignr ymm2, ymm5, ymm2, 15
//         vmovdqa ymm7, [rip + initial_min_mask]
//         vpshufb ymm7, ymm7, ymm2
//         vpcmpgtb ymm8, ymm7, ymm1
//         vmovdqa ymm7, [rip + second_min_mask]
//         vpshufb ymm7, ymm7, ymm2
//         vpcmpgtb ymm7, ymm7, ymm4
//         vpand ymm7, ymm7, ymm8
//         vpor ymm0, ymm0, ymm7
//         vmovdqa ymm1, ymm4
//         vmovdqa ymm2, ymm5
//         add rcx, 64
//         cmp rcx, rsi
//         jb process_loop
//         vptest ymm0, ymm0
//         jz result_no_error
//         mov rax, 0
//         jmp result_has_error
// result_no_error:
//         mov rax, 1
// result_has_error:
//     ":"={rax}"(out)
//     :"{rdi}"(src.as_ptr()), "{rsi}"(src.len())
//     :"ymm0","ymm1","ymm2","ymm3","ymm4","ymm5","ymm6","ymm7","ymm8","ymm9","rcx"
//     :"intel") };
//     out
// }

