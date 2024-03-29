// Divide a 16-byte vector by 2 using SIMD on x86_64.
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
fn divide_by_two_simd(v: *mut u8) {
    use std::arch::x86_64::*;

    unsafe {
        let a = _mm_loadu_si128(v as *const __m128i);
        let b = _mm_set1_epi8(2);
        let c = _mm_div_epi8(a, b);
        _mm_storeu_si128(v as *mut __m128i, c);
    }
}

// Divide a 16-byte vector by 2 using SIMD on arm64.
// I refuse to call it aarch64: https://lkml.org/lkml/2012/7/15/133
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
fn divide_by_two_simd(v: *mut u8) {
    use std::arch::aarch64::*;

    unsafe {
        let a = vld1q_u8(v);
        let b = vshrq_n_u8(a, 1);
        vst1q_u8(v, b);
    }
}

// Set the brightness of the specified image in half using SIMD. Note that this
// also affects the alpha channel, but for the background rendering this is fine.
pub fn set_brightness_half_simd(image: &mut image::RgbaImage) {
    // Get the total length.
    let len = image.len();

    // Get the number of 16 byte vectors.
    let num_vecs = len / 16;
    let remainder = len % 16;

    // Get a mutable pointer to the pixels.
    let mut pixels = image.as_mut_ptr();

    // Iterate through the vectors.
    for _ in 0..num_vecs {
        // Divide the vector by 2.
        divide_by_two_simd(pixels);

        // Increment the pointer.
        unsafe { pixels = pixels.add(16) };
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
