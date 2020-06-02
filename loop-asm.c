#![feature(asm)]

fn main() {
    let result: u64;
    unsafe { asm!("
        xor rax, rax
    ${:private}process_loop:
        nop
        add rax, 1
        cmp rax, 500
        jne ${:private}process_loop
    ": "={rax}"(result)
    :
    :
    :"intel") };
    println!("Result: {}", result);
}