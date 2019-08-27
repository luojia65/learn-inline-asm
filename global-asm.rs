#![feature(asm, global_asm)]
global_asm! { r#"
    .section	.data, ""
letter_lower_i_mask:
    .ascii "iiiiiiiiiiiiiiii"
"# }

fn main() {
    let src = b" important stuff important stuff important stuff important stuff";
    let ans = process(src);  
    println!("{:064b}", ans);
}

fn process(src: &[u8]) -> u64 {
    let ans: u64;
    unsafe { asm!("
        lea rax, [rip + letter_lower_i_mask]
        movdqu xmm1, [rax]
        movdqu xmm0, [r8]
        pcmpeqb xmm0, xmm1
        pmovmskb r9, xmm0
        mov rax, r9
        movdqu xmm0, [r8+10h]
        pcmpeqb xmm0, xmm1
        pmovmskb r9, xmm0
        shl rax, 10h
        or rax, r9
        movdqu xmm0, [r8+20h]
        pcmpeqb xmm0, xmm1
        pmovmskb r9, xmm0
        shl rax, 10h
        or rax, r9
        movdqu xmm0, [r8+30h]
        pcmpeqb xmm0, xmm1
        pmovmskb r9, xmm0
        shl rax, 10h
        or rax, r9
    ":"={rax}"(ans)
    :"{r8}"(src.as_ptr())
    :"xmm0", "xmm1", "rax" "r8", "r9"
    :"intel") };
    ans
}

