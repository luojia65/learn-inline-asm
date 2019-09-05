#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[export_name = "main"]
pub fn main() -> ! {
    let a: u32 = 0x12345678;
    let b: u32 = 0x90ABCDEF;
    if a + b > 0x87654321 {
        loop {}
    }
    loop {}
}

// #[export_name = "_start"]
// pub fn _start() -> ! { 
//     main() 
// }