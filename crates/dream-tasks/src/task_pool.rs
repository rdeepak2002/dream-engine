use std::future::Future;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_executor::{Executor, LocalExecutor};
use instant::Instant;
use once_cell::sync::Lazy;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;
use wasm_bindgen::prelude::*;

// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::*;

// #[cfg(target_arch = "wasm32")]
// pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen(module = "/js/dream-tasks.js")]
extern "C" {
    fn sleep(sleep_duration: u64);
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use std::time;
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

// static ASYNC_COMPUTE_TASK_POOL: Lazy<RwLock<AsyncComputeTaskPool>> =
//     Lazy::new(|| RwLock::new(AsyncComputeTaskPool::default()));
//
// pub fn get_async_task_pool<'a>() -> RwLockReadGuard<'static, AsyncComputeTaskPool<'a>> {
//     return ASYNC_COMPUTE_TASK_POOL.read().unwrap();
// }
//
// #[derive(Default)]
// pub struct AsyncComputeTaskPool<'a> {
//     executor: Executor<'a>,
// }

// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
// pub fn start_thread(sleep_millis: u64) {
// let mut last_update_time = Instant::now();
// loop {
//     let now = Instant::now();
//     if (now - last_update_time).as_millis() > sleep_millis as u128 {
//         get_async_task_pool().try_tick();
//         last_update_time = Instant::now();
//     }
//     if sleep_millis != 0 {
//         cfg_if::cfg_if! {
//             if #[cfg(target_arch = "wasm32")] {
//                 sleep(sleep_millis);
//             } else {
//                 thread::sleep(time::Duration::from_millis(sleep_millis));
//             }
//         }
//     }
// }
// }
//
// impl<'task> AsyncComputeTaskPool<'task> {
//     pub fn try_tick(&self) {
//         if !self.executor.is_empty() {
//             self.executor.try_tick();
//         }
//     }
//
//     pub fn spawn<T: Send + 'task>(&self, future: impl Future<Output = T> + Send + 'task) {
//         log::warn!("Spawning task (TODO: verify this is not called too many times)");
//         let task = self.executor.spawn(future);
//         task.detach();
//     }
// }
