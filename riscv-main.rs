#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[export_name = "main"]
fn main() -> ! {
    // code here
    loop {}
}

#[no_mangle]
#[inline(never)]
fn riscv_process(a: u64, b: u64) -> u64 {
    let ret: u64;
    unsafe { asm!("
    // code here
        "
    ) };
    ret
}
