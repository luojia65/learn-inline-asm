use std::arch::x86_64::*;
use std::mem::transmute;

fn main() {
    for input in 0..=0xFFu64 {
        if input.count_ones() % 2 != 0 {
            continue
        }
        let quote_mask: u64 = unsafe { _mm_cvtsi128_si64(_mm_clmulepi64_si128(
            _mm_set_epi64x(0, transmute::<_, i64>(input)),
            _mm_set1_epi8(transmute::<_, i8>(0xFFu8)),
            0,
        )) as u64 };
        println!("in:\t{:064b}", input);
        println!("out:\t{:064b}", quote_mask);
        let quote_mask = u64::wrapping_mul(input, 0xFFFF_FFFF_FFFF_FFFF);
        println!("out:\t{:064b}", quote_mask);
        println!();
    }
}
