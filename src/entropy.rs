//! 即时取熵逻辑。
//!
//! 约束是每次 `Enter` 才生成一个数字，绝不预生成。
//! 因此这里暴露的是“逐次取样”的接口，而不是批量随机 API。

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use getrandom::fill;

/// 一次按键取样得到的结果。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntropySample {
    pub digit: u8,
    pub fingerprint: String,
}

/// 取熵源抽象，便于替换和测试状态机。
pub trait EntropySource {
    fn next_digit(&mut self, throw_no: usize) -> Result<EntropySample>;
}

/// 默认系统熵源。
pub struct SystemEntropy;

impl EntropySource for SystemEntropy {
    fn next_digit(&mut self, throw_no: usize) -> Result<EntropySample> {
        generate_timing_digit(throw_no)
    }
}

/// 在按键发生的当下，混合纳秒时间戳与操作系统安全随机数。
pub fn generate_timing_digit(throw_no: usize) -> Result<EntropySample> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let mut os_entropy = [0u8; 16];
    fill(&mut os_entropy)?;

    let mut mixed = (now as u64) ^ ((now >> 64) as u64) ^ (throw_no as u64).rotate_left(17);
    for chunk in os_entropy.chunks_exact(8) {
        mixed ^= u64::from_le_bytes(chunk.try_into().expect("8-byte chunk"));
        mixed = splitmix64(mixed);
    }

    let digit = if mixed & 1 == 0 { 2 } else { 3 };
    let fingerprint = format!("0x{:016X}", mixed);

    Ok(EntropySample { digit, fingerprint })
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::generate_timing_digit;

    #[test]
    fn generated_digits_stay_in_yarrow_domain() {
        for throw_no in 1..=32 {
            let sample = generate_timing_digit(throw_no).expect("entropy sample");
            assert!(matches!(sample.digit, 2 | 3));
            assert!(sample.fingerprint.starts_with("0x"));
        }
    }
}
