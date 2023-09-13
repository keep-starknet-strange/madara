use std::sync::{Arc, Mutex};

use pretty_assertions::assert_eq;

use super::Retry;
use crate::test_utils::retry::get_test_config;

struct Worker {
    // Number of times the worker was called. Updated in every call to work.
    number_of_calls: Arc<Mutex<usize>>,
    // Number of times the worker returns errors before it returns ok.
    number_of_errors: Arc<usize>,
}

impl Worker {
    fn new(number_of_errors: usize) -> Self {
        Worker {
            number_of_calls: Arc::new(Mutex::new(0)),
            number_of_errors: Arc::new(number_of_errors),
        }
    }

    fn get_last_attempt(&self) -> usize {
        *self.number_of_calls.lock().unwrap()
    }

    async fn work(&self) -> Result<(), &str> {
        let mut number_of_calls = self.number_of_calls.lock().unwrap();
        *number_of_calls += 1;

        if *number_of_calls <= *self.number_of_errors { Err("Some error.") } else { Ok(()) }
    }
}

#[tokio::test]
async fn fail_on_all_attempts() {
    let config = get_test_config();
    let worker = Worker::new(10);
    Retry::new(&config).start(|| worker.work()).await.unwrap_err();
    assert_eq!(worker.get_last_attempt(), 5);
}

#[tokio::test]
async fn success_on_third_attempt() {
    let config = get_test_config();
    let worker = Worker::new(2);
    Retry::new(&config).start(|| worker.work()).await.unwrap();
    assert_eq!(worker.get_last_attempt(), 3);
}
