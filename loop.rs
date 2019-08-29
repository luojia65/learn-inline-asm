#![feature(asm)]

fn main() {
    unsafe { asm!("
        xor rax, rax
    .process_loop:
        nop
        add rax, 1
        cmp rax, 500
        jne .process_loop
    ":::"rax":"intel"
    ) }
}