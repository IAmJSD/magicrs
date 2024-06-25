// Divide a 64-byte vector by 2 using neon on arm64.
// I refuse to call it aarch64: https://lkml.org/lkml/2012/7/15/133
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
unsafe fn divide_by_two_simd(v: *mut u8) {
    use std::arch::aarch64::*;

    let mut regs = vld1q_u8_x4(v);
    regs.0 = vshrq_n_u8(regs.0, 1);
    regs.1 = vshrq_n_u8(regs.1, 1);
    regs.2 = vshrq_n_u8(regs.2, 1);
    regs.3 = vshrq_n_u8(regs.3, 1);
    vst1q_u8_x4(v, regs);
}

// Divide a 32-byte vector by 2 using AVX2 on x86_64.
#[cfg(target_arch = "x86_64")]
unsafe fn divide_by_two_simd_avx2(v: *mut u8) {
    use std::arch::x86_64::*;

    // Load the 32-byte vector into a 256-bit registers.
    let mut reg = _mm256_loadu_si256(v as *const __m256i);

    // Perform the bitwise shift right operation by 1 on each 128-bit register.
    reg = _mm256_srli_epi16(reg, 1);

    // Mask out the upper 7 bits to ensure no overflow into adjacent bytes.
    let mask = _mm256_set1_epi8(0x7F);
    reg = _mm256_and_si256(reg, mask);

    // Store the result back into the original vector.
    _mm256_storeu_si256(v as *mut __m256i, reg);
}

// Divide a 32-byte vector by 2 using SSE2 on x86_64.
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
unsafe fn divide_by_two_simd_fallback(v: *mut u8) {
    use std::arch::x86_64::*;

    // Load the 32-byte vector into 2 128-bit registers.
    let mut reg1 = _mm_loadu_si128(v as *const __m128i);
    let mut reg2 = _mm_loadu_si128(v.add(16) as *const __m128i);

    // Perform the bitwise shift right operation by 1 on each 128-bit register.
    reg1 = _mm_srli_epi16(reg1, 1);
    reg2 = _mm_srli_epi16(reg2, 1);

    // Mask out the upper 7 bits to ensure no overflow into adjacent bytes.
    let mask = _mm_set1_epi8(0x7F);
    reg1 = _mm_and_si128(reg1, mask);
    reg2 = _mm_and_si128(reg2, mask);

    // Store the result back into the original vector.
    _mm_storeu_si128(v as *mut __m128i, reg1);
    _mm_storeu_si128(v.add(16) as *mut __m128i, reg2);
}

// Defines the width of the vector.
const VECTOR_WIDTH: usize = if cfg!(all(target_arch = "aarch64", target_feature = "neon")) {
    64
} else {
    32
};

// Set the brightness of the specified image in half using SIMD. Note that this
// also affects the alpha channel, but for the background rendering this is fine.
// Also gets the average of the RGB channels.
pub fn set_brightness_half_simd(image: &mut image::RgbaImage) {
    // Get the total length.
    let len = image.len();

    // Get the number of maximum CPU width vectors.
    let num_vecs = len / VECTOR_WIDTH;
    let remainder = len % VECTOR_WIDTH;

    // Get a mutable pointer to the pixels.
    let mut pixels = image.as_mut_ptr();

    // On x86_64, check if we can use AVX2. This is basically every x86 CPU since 2013.
    #[cfg(target_arch = "x86_64")]
    let has_avx2 = is_x86_feature_detected!("avx2");

    // Iterate through the vectors and divide them by 2.
    macro_rules! iterate_vectors {
        ($method_name:ident) => {
            for _ in 0..num_vecs {
                unsafe {
                    // SAFETY: The pointer is valid and aligned as per the instruction types
                    // and the specific lengths set in the constant above.
                    $method_name(pixels);
                    pixels = pixels.add(VECTOR_WIDTH);
                }
            }
        };
    }
    #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
    iterate_vectors!(divide_by_two_simd);
    #[cfg(target_arch = "x86_64")]
    if has_avx2 {
        iterate_vectors!(divide_by_two_simd_avx2);
    } else {
        iterate_vectors!(divide_by_two_simd_fallback);
    }

    // Iterate through the remainder.
    for _ in 0..remainder {
        // Decrement the pixel by half.
        let pixel = unsafe { &mut *pixels };
        *pixel = *pixel / 2;

        // Increment the pointer.
        unsafe { pixels = pixels.add(1) };
    }
}
