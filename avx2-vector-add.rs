#![feature(asm, global_asm)]

global_asm! { r#"
    .section    .data, ""
    .p2align    4
x_indexes:
    .quad 0x0000000100000000
    .quad 0x0000000300000002
    .quad 0x0000000500000004
    .quad 0x0000000700000006
"# }

struct Point(f32, f32);

fn main() {
    let a = &[Point(1.0, 2.0), Point(10.0, 20.0)];
    let b = &[Point(3.0, 6.0), Point(30.0, 60.0)];
    let _ = process(a, b);
}

//todo
fn process(a: &[Point], _b: &[Point]) {
    let out = [0f32; 8];
    unsafe { asm!("
        vmovdqa ymm0, [rip + x_indexes]
        vpxor ymm2, ymm2, ymm2
        vgatherdps ymm1, [rdi + 4 * ymm0], ymm2
        vmovdqu [rax], ymm1
    ":
    :"{rdi}"(a.as_ptr()),"{rax}"(&out as *const _)
    ::"intel") };
    for i in 0..8 {
        print!("{} ", out[i])
    }
    println!()
}