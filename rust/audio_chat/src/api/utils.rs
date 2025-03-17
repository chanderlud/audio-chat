#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// multiplies each element in the slice by the factor, clamping result between -1 and 1
pub(crate) fn mul(frame: &mut [f32], factor: f32) {
    if is_x86_feature_detected!("avx2") {
        unsafe {
            mul_simd_avx2(frame, factor);
        }
    } else {
        for p in frame.iter_mut() {
            *p *= factor;
            *p = p.clamp(-1_f32, 1_f32);
        }
    }
}

/// optimized mul for avx2
#[target_feature(enable = "avx2")]
unsafe fn mul_simd_avx2(frame: &mut [f32], factor: f32) {
    let len = frame.len();
    let mut i = 0;

    let factor_vec = _mm256_set1_ps(factor);
    let min_vec = _mm256_set1_ps(-1_f32);
    let max_vec = _mm256_set1_ps(1_f32);

    while i + 8 <= len {
        let mut chunk = _mm256_loadu_ps(frame.as_ptr().add(i)); // load
        chunk = _mm256_mul_ps(chunk, factor_vec); // multiply
        chunk = _mm256_max_ps(min_vec, _mm256_min_ps(max_vec, chunk)); // clamp
        _mm256_storeu_ps(frame.as_mut_ptr().add(i), chunk); // write
        i += 8;
    }
}

/// calculates the RMS of the frame (loop is unrolled for optimization)
pub(crate) fn calculate_rms(data: &[f32]) -> f32 {
    let mut sum1 = 0.0;
    let mut sum2 = 0.0;
    let mut sum3 = 0.0;
    let mut sum4 = 0.0;

    let mut i = 0;
    while i + 3 < data.len() {
        sum1 += data[i] * data[i];
        sum2 += data[i + 1] * data[i + 1];
        sum3 += data[i + 2] * data[i + 2];
        sum4 += data[i + 3] * data[i + 3];
        i += 4;
    }

    let mean_of_squares = (sum1 + sum2 + sum3 + sum4) / data.len() as f32;
    mean_of_squares.sqrt()
}

/// converts a decibel value to a multiplier
pub(crate) fn db_to_multiplier(db: f32) -> f32 {
    10_f32.powf(db / 20_f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mul() {
        let frame = crate::api::audio_chat::tests::dummy_frame();
        let mut scalar_frame = frame.clone();
        let mut simd_avx2_frame = frame.clone();

        mul(&mut scalar_frame, 2_f32);
        unsafe {
            mul_simd_avx2(&mut simd_avx2_frame, 2.0_f32);
        }

        assert_eq!(scalar_frame, simd_avx2_frame);
    }
}
