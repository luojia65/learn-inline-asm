#![feature(asm)]

fn main() {
    let src = b" important stuff important stuff important stuff important stuff";
    let ans = process(src);  
    println!("{:064b}", ans);
}

#[inline(never)]
fn process(src: &[u8]) -> u64 {
    let msk = b"iiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii";
    let ans: u64;
    unsafe { asm!("
        vmovdqu ymm0, [r8]
        vmovdqu ymm1, [r9]
        vpcmpeqb ymm0, ymm0, ymm1
        vpmovmskb r10, ymm0
        vmovdqu ymm0, [r8]
        vpcmpeqb ymm0, ymm0, ymm1
        vpmovmskb rax, ymm0
        shl rax, 20h
        or rax, r10
    ":"={rax}"(ans)
    :"{r8}"(src.as_ptr()), "{r9}"(msk.as_ptr())
    :"xmm0", "xmm1", "rax", "r8", "r9", "r10"
    :"intel") };
    ans
}
