#![feature(asm)]

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[export_name = "main"]
fn main() {
    let _value = riscv_add(100002, 200001);
}

#[no_mangle]
#[inline(never)]
fn riscv_add(a: u64, b: u64) -> u64 {
    let ret: u64;
    unsafe { asm!("
        add $0, $1, $2
        ":"=r"(ret)
        :"r"(a), "r"(b)
    ) };
    ret
}
