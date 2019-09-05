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
    let a: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8];
    let b: &[u32] = &[2, 2, 3, 3, 4, 4, 5, 5];
    let c: &[u32] = &[0, 0, 0, 0, 0, 0, 0, 0];
    // calculate c = a + b
    unsafe { asm!("
addint32:
        vsetvli t0, $0, e32
        vlw.v v0, ($1)
        sub $0, $0, t0
        slli t0, t0, 2
        add $1, $1, t0
        vlw.v v1, ($2)
        add $2, $2, t0
        vadd.vv v2, v0, v1
        vsw.v v2, ($3)
        add $3, $3, t0
        bnez $0, addint32
        ":
        :"r"(c.len()), "r"(a.as_ptr()),
        "r"(b.as_ptr()), "r"(c.as_ptr())
        :"t0", "v0", "v1", "v2"
    ) };
}
