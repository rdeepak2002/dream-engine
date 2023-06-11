use std::future::Future;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use async_executor::Executor;
use once_cell::sync::Lazy;

// use rayon::iter::IntoParallelRefIterator;
// use rayon::prelude::*;
// use wasm_bindgen::prelude::*;
// pub use wasm_bindgen_rayon::init_thread_pool;

// #[wasm_bindgen]
// pub fn sum_of_squares(numbers: &[i32]) -> i32 {
//     numbers.par_iter().map(|x| x * x).sum()
// }

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

impl<'task> AsyncComputeTaskPool<'task> {
    pub fn start_thread(&self, sleep_millis: u64) {
        rayon::spawn(move || {
            log::warn!("starting background task");
            get_task_pool().try_tick();
            // thread::sleep(time::Duration::from_millis(sleep_millis));
            cfg_if::cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                } else {
                    thread::sleep(time::Duration::from_millis(sleep_millis));
                }
            }
        });
        // cfg_if::cfg_if! {
        //     if #[cfg(target_arch = "wasm32")] {
        //         // log::error!("TODO: start thread to execute background tasks");
        //     } else {
        //         thread::Builder::new()
        //         .name("child thread 1".to_string())
        //         .spawn(move || loop {
        //             get_task_pool().try_tick();
        //             thread::sleep(time::Duration::from_millis(sleep_millis));
        //         })
        //         .expect("unable to create child thread 1");
        //     }
        // }
    }

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
