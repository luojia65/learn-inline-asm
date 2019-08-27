#![feature(asm, global_asm)]
global_asm! { r#"
    .section	.data, ""
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

fn process(src: &[u8]) -> bool {
    let ans: bool;
    unsafe { asm!("
        movdqu xmm2, [rip + var_high_bit]
        movdqu xmm0, [r8]
        movdqu xmm1, [r8+10h]
        por xmm0, xmm1
        ptest xmm0, xmm2
        jz ascii_validate_success
        mov rax, 0
        jmp ascii_validate_end
    ascii_validate_success:
        mov rax, 1
    ascii_validate_end:
    ":"={rax}"(ans)
    :"{r8}"(src.as_ptr())
    :"rax","r8","xmm0","xmm1","xmm2"
    :"intel") };
    ans
}
