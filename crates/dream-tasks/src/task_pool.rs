use std::sync::RwLock;

use async_executor::Executor;
use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;

static EXECUTOR: RwLock<Executor> = RwLock::new(Executor::new());
static MULTITHREADING_ENABLED: RwLock<bool> = RwLock::new(true);

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
    let executor = EXECUTOR
        .try_read()
        .expect("Unable to acquire lock on executor");
    if !executor.is_empty() {
        executor.try_tick();
    }
}

pub fn set_multithreading(multithreading_enabled: bool) {
    let mut multithreading_enabled_lock = MULTITHREADING_ENABLED
        .try_write()
        .expect("Unable to acquire lock on multithreading flag");
    *multithreading_enabled_lock = multithreading_enabled;
}

pub fn start_worker_thread() {
    // TODO: instead of just one executor, we can have multiple using Vec<RwLock>
    rayon::spawn(|| loop {
        let executor = EXECUTOR.try_read().expect("Unable to lock executor");
        if !executor.is_empty() {
            executor.try_tick();
        }
        sleep_universal(100);
    });
}

pub fn spawn<F>(func: F)
where
    F: FnOnce() + Send + 'static,
{
    let multithreaded = MULTITHREADING_ENABLED
        .try_read()
        .expect("Unable to acquire lock on multithreading flag");

    if multithreaded.eq(&true) {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                // on web build we have to send tasks to executor(s) to be executed on web worker
                spawn_web_task(func);
            } else {
                // on non-web build we can just normally spawn tasks
                rayon::spawn(func);
            }
        }
    } else {
        spawn_web_task(func);
    }
}

pub fn spawn_web_task<F>(func: F)
where
    F: FnOnce() + Send + 'static,
{
    loop {
        if let Ok(executor) = EXECUTOR.try_read() {
            let task = executor.spawn(async move {
                func();
            });
            task.detach();
            break;
        } else {
            log::debug!("Attempting to acquire lock for executor again");
        }
    }
}
