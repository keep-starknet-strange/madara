use crate::retry::RetryConfig;

pub const MAX_RETRIES: usize = 4;

pub fn get_test_config() -> RetryConfig {
    // Taking the fastest config possible (except for MAX_RETRIES which we want to be a bit bigger
    // to test the functionality).
    RetryConfig { retry_base_millis: 0, retry_max_delay_millis: 0, max_retries: MAX_RETRIES }
}
