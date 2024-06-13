// Divide a 16-byte vector by 2 using SIMD on x86_64.
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
unsafe fn divide_by_two_simd(v: *mut u8) {
    use std::arch::x86_64::*;

    let a = _mm_loadu_si128(v as *const __m128i);
    let b = _mm_srli_epi16(a, 1);
    _mm_storeu_si128(v as *mut __m128i, b);
}

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

// Defines the width of the vector.
const VECTOR_WIDTH: usize = if cfg!(all(target_arch = "aarch64", target_feature = "neon")) {
    64
} else {
    16
};

// Set the brightness of the specified image in half using SIMD. Note that this
// also affects the alpha channel, but for the background rendering this is fine.
pub fn set_brightness_half_simd(image: &mut image::RgbaImage) {
    // Get the total length.
    let len = image.len();

    // Get the number of 16 byte vectors.
    let num_vecs = len / VECTOR_WIDTH;
    let remainder = len % VECTOR_WIDTH;

    // Get a mutable pointer to the pixels.
    let mut pixels = image.as_mut_ptr();

    // Iterate through the vectors.
    for _ in 0..num_vecs {
        // Divide the vector by 2.
        // SAFETY: The pointer is valid and aligned.
        unsafe { divide_by_two_simd(pixels) }

        // Increment the pointer.
        unsafe { pixels = pixels.add(VECTOR_WIDTH) };
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
