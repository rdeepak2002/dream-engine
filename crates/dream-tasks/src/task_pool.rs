use std::future::Future;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_executor::Executor;
use once_cell::sync::Lazy;
use rayon::iter::IntoParallelRefIterator;
use rayon::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
pub use wasm_bindgen_rayon::init_thread_pool;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn sum_of_squares(numbers: &[i32]) -> i32 {
    numbers.par_iter().map(|x| x * x).sum()
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use std::time;
    } else {
        use std::{thread, time};
    }
}

static ASYNC_COMPUTE_TASK_POOL: Lazy<RwLock<AsyncComputeTaskPool>> =
    Lazy::new(|| RwLock::new(AsyncComputeTaskPool::default()));

pub fn get_task_pool<'a>() -> RwLockReadGuard<'static, AsyncComputeTaskPool<'a>> {
    return ASYNC_COMPUTE_TASK_POOL.read().unwrap();
}

pub fn get_task_pool_to_write<'a>() -> RwLockWriteGuard<'static, AsyncComputeTaskPool<'a>> {
    return ASYNC_COMPUTE_TASK_POOL.write().unwrap();
}

#[derive(Default)]
pub struct AsyncComputeTaskPool<'a> {
    executor: Executor<'a>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn start_thread(sleep_millis: u64) {
    rayon::spawn(move || {
        log::warn!("starting background task");
        log::warn!("TODO: limit background task execution time for wasm - maybe on worker JS side just have a timeout function that sleeps");
        loop {
            get_task_pool().try_tick();
            // thread::sleep(time::Duration::from_millis(sleep_millis));
            cfg_if::cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                } else {
                    thread::sleep(time::Duration::from_millis(sleep_millis));
                }
            }
        }
    });
}

impl<'task> AsyncComputeTaskPool<'task> {
    pub fn try_tick(&self) {
        if !self.executor.is_empty() {
            self.executor.try_tick();
        }
    }

    pub fn spawn<T: Send + 'task>(&self, future: impl Future<Output = T> + Send + 'task) {
        let task = self.executor.spawn(async move { future.await });
        task.detach();
    }
}
