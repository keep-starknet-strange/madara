#[cfg(test)]
#[path = "retry_test.rs"]
mod retry_test;

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::iter::Take;
use std::time::Duration;

use papyrus_config::dumping::{ser_param, SerializeConfig};
use papyrus_config::{ParamPath, SerializedParam};
use serde::{Deserialize, Serialize};
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::{Action, Condition, RetryIf};
use tracing::debug;

/// A configuration for the retry mechanism.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct RetryConfig {
    /// The initial waiting time in milliseconds.
    pub retry_base_millis: u64,
    /// The maximum waiting time in milliseconds.
    pub retry_max_delay_millis: u64,
    /// The maximum number of retries.
    pub max_retries: usize,
}

impl SerializeConfig for RetryConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([
            ser_param(
                "retry_base_millis",
                &self.retry_base_millis,
                "Base waiting time after a failed request. After that, the time increases \
                 exponentially.",
            ),
            ser_param(
                "retry_max_delay_millis",
                &self.retry_max_delay_millis,
                "Max waiting time after a failed request.",
            ),
            ser_param(
                "max_retries",
                &self.max_retries,
                "Maximum number of retries before the node stops retrying.",
            ),
        ])
    }
}

/// A utility for retrying actions with a configurable backoff and error filter. Uses an
/// [`ExponentialBackoff`] strategy.
pub struct Retry {
    strategy: Take<ExponentialBackoff>,
}

impl Retry {
    pub fn new(config: &RetryConfig) -> Self {
        Retry {
            strategy: ExponentialBackoff::from_millis(config.retry_base_millis)
                .max_delay(Duration::from_millis(config.retry_max_delay_millis))
                .take(config.max_retries),
        }
    }

    fn log_condition<E, C>(err: &E, condition: &mut C) -> bool
    where
        E: Debug,
        C: Condition<E>,
    {
        if condition.should_retry(err) {
            debug!("Received error {:?}, retrying.", err);
            true
        } else {
            debug!("Received error {:?}, not retrying.", err);
            false
        }
    }

    pub async fn start<I, E, A>(&self, action: A) -> Result<I, E>
    where
        E: Debug,
        A: Action<Item = I, Error = E>,
    {
        self.start_with_condition(action, |_: &_| true).await
    }

    pub async fn start_with_condition<I, E, A, C>(
        &self,
        action: A,
        mut condition: C,
    ) -> Result<I, E>
    where
        E: Debug,
        A: Action<Item = I, Error = E>,
        C: Condition<E> + Send,
    {
        let condition: Box<dyn Send + FnMut(&E) -> bool> =
            Box::new(|err| Self::log_condition(err, &mut condition));
        RetryIf::spawn(self.strategy.clone(), action, condition).await
    }
}
