use std::time::Duration;

use log::error;
use rand::{rng, Rng};

const DEFAULT_SLEEP_INITIAL_DURATION: Duration = Duration::from_secs(1);
const DEFAULT_SLEEP_BACKOFF_FACTOR: f64 = 1.2;
const DEFAULT_JITTER: Duration = Duration::from_millis(800);

#[allow(clippy::module_name_repetitions)]
pub struct RetryConfig {
    sleep_duration: Duration,
    backoff_factor: f64,
    jitter: Duration,
}

impl RetryConfig {
    pub fn new(sleep_duration: Duration, backoff_factor: f64, jitter: Duration) -> Self {
        Self {
            sleep_duration,
            backoff_factor,
            jitter,
        }
    }

    pub fn sleep_after_error(&mut self) {
        std::thread::sleep(self.sleep_duration);
        self.sleep_duration = self.sleep_duration.mul_f64(self.backoff_factor)
            + rng().random_range(Duration::ZERO..self.jitter);
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new(
            DEFAULT_SLEEP_INITIAL_DURATION,
            DEFAULT_SLEEP_BACKOFF_FACTOR,
            DEFAULT_JITTER,
        )
    }
}

pub fn execute_with_retries<T, E: std::fmt::Debug>(
    mut func: impl FnMut() -> Result<T, E>,
    mut retry_config: RetryConfig,
    max_retries: usize,
) -> Result<T, E> {
    let mut exec_result = func();
    for _ in 0..max_retries {
        if exec_result.is_ok() {
            return exec_result;
        }
        retry_config.sleep_after_error();
        exec_result = func();
    }
    if let Err(ref e) = exec_result {
        error!("Operation failed after {max_retries} retries: {e:?}");
    }

    exec_result
}
