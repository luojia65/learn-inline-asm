#![feature(asm)]

fn main() {
    let src = b" important stuff important stuff important stuff important stuff";
    let ans = process(src);  
    println!("{:064b}", ans);
}

fn process(src: &[u8]) -> u64 {
    let msk = b"iiiiiiiiiiiiiiii";
    let ans: u64;
    unsafe { asm!("
        movdqa  xmm0, [r8]
        movdqa  xmm1, [r9]
        pcmpeqb xmm0, xmm1
        pmovmskb    r10, xmm0
        mov r11, r10
        movdqa  xmm0, [r8+10h]
        pcmpeqb xmm0, xmm1
        pmovmskb    r10, xmm0
        shl r10, 10h
        or  r11, r10
        movdqa  xmm0, [r8+20h]
        pcmpeqb xmm0, xmm1
        pmovmskb    r10, xmm0
        shl r10, 20h
        or  r11, r10
        movdqa  xmm0, [r8+30h]
        pcmpeqb xmm0, xmm1
        pmovmskb    r10, xmm0
        shl r10, 30h
        or  r11, r10
    ":"={r11}"(ans)
    :"{r8}"(src.as_ptr()), "{r9}"(msk.as_ptr())
    :"xmm0", "xmm1", "r8", "r9", "r10", "r11"
    :"intel") };
    ans
}
