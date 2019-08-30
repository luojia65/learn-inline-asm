#![feature(asm, global_asm)]
use std::arch::x86_64::*;
use std::mem::transmute;

global_asm! { r#"
    .section    .data, ""
    .p2align    4
"# }

fn main() {
    let _ = process();
}

fn process() {
    unsafe { asm!("

    ") };
}