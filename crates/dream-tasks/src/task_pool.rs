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
pub fn complete_task() {
    let executor = EXECUTOR.lock().expect("Unable to acquire lock on executor");
    if !executor.is_empty() {
        executor.try_tick();
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
        rayon::spawn(func);
    } else {
        loop {
            if let Ok(executor) = EXECUTOR.try_lock() {
                let task = executor.spawn(async move {
                    func();
                });
                task.detach();
                break;
            } else {
                log::warn!("Attempting to acquire lock for executor again");
            }
        }
    }
}
