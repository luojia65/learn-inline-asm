#![feature(asm, global_asm)]

global_asm! { r#"
    .section    .data, ""
    .p2align    4
input_1:
    .quad 0xBB000000000000AA
    .quad 0xCC000000000000BB
    .quad 0xDD000000000000CC
    .quad 0xEE000000000000DD
input_2:
    .quad 0x22FFFFFFFFFFFF11
    .quad 0x44FFFFFFFFFFFF33
    .quad 0x66FFFFFFFFFFFF55
    .quad 0x88FFFFFFFFFFFF77
"# }

fn main() {
    let _ = process();
}

fn process() {
    let out = [0u64; 4];
    unsafe { asm!("
        vmovdqa ymm1, [rip + input_1]
        vmovdqa ymm2, [rip + input_2]
        vperm2i128 ymm0, ymm1, ymm2, 0x21
        // vpalignr ymm0, ymm2, ymm0, 14
        vmovdqu [rax], ymm0
    ":
    :"{rax}"(out.as_ptr())
    ::"intel") };    
    println!("{:016X}{:016X}{:016X}{:016X}", out[3], out[2], out[1], out[0]);
}