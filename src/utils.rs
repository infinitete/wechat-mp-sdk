use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

const MAX_BACKOFF_MS: u64 = 30_000;

pub(crate) fn jittered_delay(base_delay_ms: u64, attempt: u32) -> Duration {
    let backoff_multiplier = 1u64.checked_shl(attempt).unwrap_or(u64::MAX);
    let base = base_delay_ms.saturating_mul(backoff_multiplier);

    let jitter_upper_bound = base / 2;
    let jitter = if jitter_upper_bound == 0 {
        0
    } else {
        let current_time_nanos = Instant::now().elapsed().as_nanos();
        let mut hasher = DefaultHasher::new();
        (attempt as u64).hash(&mut hasher);
        current_time_nanos.hash(&mut hasher);
        hasher.finish() % jitter_upper_bound
    };

    let total_delay_ms = base.saturating_add(jitter).min(MAX_BACKOFF_MS);
    Duration::from_millis(total_delay_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jittered_delay_bounds() {
        let base_delay_ms = 100;
        for attempt in 0..9 {
            let base = base_delay_ms * 2u64.pow(attempt);
            let delay = jittered_delay(base_delay_ms, attempt);
            assert!(delay >= Duration::from_millis(base));
            assert!(delay <= Duration::from_millis(base + (base / 2)));
        }
    }

    #[test]
    fn test_jittered_delay_max_cap() {
        let delay = jittered_delay(1_000, 20);
        assert!(delay <= Duration::from_secs(30));
    }

    #[test]
    fn test_jittered_delay_handles_overflow_safely() {
        let delay = jittered_delay(u64::MAX, u32::MAX);
        assert_eq!(delay, Duration::from_secs(30));
    }

    #[test]
    fn test_jittered_delay_varies_by_attempt() {
        let delay_a = jittered_delay(100, 0);
        let delay_b = jittered_delay(100, 1);
        assert_ne!(delay_a, delay_b);
    }
}
