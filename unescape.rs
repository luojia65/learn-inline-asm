use std::arch::x86_64::*;
use std::mem::transmute;

fn main() {
    const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
    const ODD_BITS: u64 = 0xAAAA_AAAA_AAAA_AAAA;
    let source = br"\\\\\;;;;\\\;\\\\\\\;;;;\\\;\;\\\\\\\;;;;;;;;;;;;;;;;;;;\\\;\;\\";
    let backslash_mask = unsafe { _mm256_set1_epi8(transmute(b'\\')) };
    let backslashes = unsafe { 
        let source_lo32 = _mm256_loadu_si256(source.as_ptr() as *const _);
        let lo32_eq = _mm256_cmpeq_epi8(source_lo32, backslash_mask);
        let lo32: u32 = transmute(_mm256_movemask_epi8(lo32_eq));
        let source_hi32 = _mm256_loadu_si256(source.as_ptr().add(32) as *const _);
        let hi32_eq = _mm256_cmpeq_epi8(source_hi32, backslash_mask);
        let hi32: u32 = transmute(_mm256_movemask_epi8(hi32_eq));
        ((hi32 as u64) << 32) | lo32 as u64
    };
    for &ch in source.iter().rev() {
        print!("{}", char::from(ch))
    }
    println!();
    println!("{:064b}", backslashes);
    let head = backslashes & !(backslashes << 1);
    let even_head = head & EVEN_BITS;
    let odd_head = head & ODD_BITS;
    let even_tail = u64::wrapping_add(backslashes, even_head);
    let even_tail = even_tail & !backslashes;
    let odd_tail = u64::wrapping_add(backslashes, odd_head);
    let odd_tail = odd_tail & !backslashes;
    let even_includes = u64::wrapping_sub(even_tail, even_head);
    let odd_includes = u64::wrapping_sub(odd_tail, odd_head);
    let even_bit_backslashes = backslashes & EVEN_BITS;
    let even_backslashes = even_bit_backslashes & even_includes;
    let odd_bit_backslashes = backslashes & ODD_BITS;
    let odd_backslashes = odd_bit_backslashes & odd_includes;
    let result = even_backslashes | odd_backslashes;
    println!("{:064b}", result);
    // We may save the result using compress or compress-store method
    // provided by x86_64's AVX-512 instruction set.
    // If the result should be bitwise negated before masking, use and-not
    // instruction instead of and when calculating `even_backslashes` and 
    // `odd_backslashes` variants.
    let tail = u64::wrapping_add(head, backslashes);
    println!("{:064b}", tail);
    // Use AVX-512 byte shuffle instruction with `tail` variant as a mask.
    // This filters and replaces `\n` etc into unescaped value.
}
