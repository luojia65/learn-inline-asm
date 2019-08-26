#![feature(asm, global_asm)]
global_asm! { r#"
var_high_bit:
    .long 0x80808080
    .long 0x80808080
    .long 0x80808080
    .long 0x80808080
"# }

fn main() {
    let src = b" important stuff important stuff important stuff important stuff";
    let ans = process(src);  
    println!("{:?}", ans);
}

fn process(src: &[u8]) -> u64 {
    let ans: u64 = 1;
    unsafe { asm!("
        movdqu xmm2, [r9]
        movdqu xmm0, [r8]
        movdqu xmm1, [r8+10h]
        por xmm0, xmm1
        ptest xmm0, xmm2
    ":
    :"{r8}"(src.as_ptr())
    :
    :"intel") };
    ans
}
