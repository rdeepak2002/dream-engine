use std::sync::Mutex;

use async_executor::Executor;
use wasm_bindgen::prelude::*;

static EXECUTOR: Mutex<Executor> = Mutex::new(Executor::new());
static MULTITHREADING_ENABLED: Mutex<bool> = Mutex::new(true);

#[wasm_bindgen(module = "/js/dream-tasks.js")]
extern "C" {
    fn sleep(sleep_duration: u64);
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
    } else {
        use std::{thread, time};
    }
}

pub fn sleep_universal(sleep_millis: u64) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            sleep(sleep_millis);
        } else {
            thread::sleep(time::Duration::from_millis(sleep_millis));
        }
    }
}

// for web build to complete tasks in background when running in non-multithreaded environment
pub async fn complete_task() {
    let executor = EXECUTOR.lock().expect("Unable to acquire lock on executor");
    if !executor.is_empty() {
        log::debug!("Running async task on async executor");
        executor.tick().await;
    }
}

pub fn set_multithreading(multithreading_enabled: bool) {
    let mut multithreading_enabled_lock = MULTITHREADING_ENABLED
        .lock()
        .expect("Unable to acquire lock on multithreading flag");
    *multithreading_enabled_lock = multithreading_enabled;
}

pub fn spawn<F>(func: F)
where
    F: FnOnce() + Send + 'static,
{
    let multithreaded = MULTITHREADING_ENABLED
        .lock()
        .expect("Unable to acquire lock on multithreading flag");

    if multithreaded.eq(&true) {
        log::debug!("Queueing task on thread pool");
        rayon::spawn(func);
        // occurs when we freshly open this in new incognito tab
        log::debug!("Done queueing task on thread pool");
    } else {
        log::debug!("Queueing async task on async executor");
        let executor = EXECUTOR.lock().expect("Unable to acquire lock on executor");
        let task = executor.spawn(async move {
            func();
        });
        task.detach();
    }
}
