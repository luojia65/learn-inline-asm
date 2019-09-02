#![feature(asm, global_asm)]

global_asm! { r#"
    .section    .data, ""
    .p2align    4
x_indexes:
    .quad 0x0000000400000000
    .quad 0x0000000C00000008
    .quad 0x0000001400000010
    .quad 0x0000001C00000018
high_bit_one:
    .long 0x80000000
"# }

struct Point(f32, f32);

fn main() {
    let a = &[
        Point(1.0, 2.0), Point(10.0, 20.0), Point(1.0, 2.0), Point(10.0, 20.0), 
        Point(1.0, 2.0), Point(10.0, 20.0), Point(1.0, 2.0), Point(10.0, 20.0), 
    ];
    let b = &[
        Point(3.0, 6.0), Point(30.0, 60.0), Point(3.0, 6.0), Point(30.0, 60.0), 
        Point(3.0, 6.0), Point(30.0, 60.0), Point(3.0, 6.0), Point(30.0, 60.0), 
    ];
    let _ = process(a, b);
}

//todo
fn process(a: &[Point], b: &[Point]) {
    let out = [0f32; 8];
    unsafe { asm!("
        vmovdqa ymm0, [rip + x_indexes]
        vbroadcastss ymm1, [rip + high_bit_one]
        vgatherdps ymm2, [rdi + 2 * ymm0], ymm1
        vbroadcastss ymm1, [rip + high_bit_one]
        vgatherdps ymm3, [rsi + 2 * ymm0], ymm1
        vaddps ymm2, ymm2, ymm3
        vmovdqu [rax], ymm2
    ":
    :"{rdi}"(a.as_ptr()),"{rsi}"(b.as_ptr()),"{rax}"(&out as *const _)
    ::"intel") };
    for i in 0..8 {
        print!("{} ", out[i])
    }
    println!()
}