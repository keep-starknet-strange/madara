use crate::l1_messages::error::L1MessagesError;

pub struct L1MessagesWorkerConfig {}
pub struct L1MessagesWorker {}

pub(crate) const LOG_TARGET: &str = "node::service::L1MessagesWorker";

use std::thread;
use std::time::Duration;

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $pattern:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: LOG_TARGET,
			concat!("⟠ ", $pattern), $(, $values)*)
	};
}

pub async fn run_worker(config: L1MessagesWorkerConfig) {
    log!(info, "L1 Messages Worker");
    thread::sleep(Duration::from_millis(1000));
}
