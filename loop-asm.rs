#![feature(asm)]

fn main() {
    unsafe { asm!("
        xor rax, rax
	.p2align	4, 0x90
    .process_loop:  
        nop
        add rax, 1
        cmp rax, 500
        jne .process_loop
    ":
    :
    :"rax"
    :"intel") };
}
