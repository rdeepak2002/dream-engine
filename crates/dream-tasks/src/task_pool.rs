use std::future::Future;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{thread, time};

use async_executor::Executor;
use crossbeam_channel::unbounded;
use once_cell::sync::Lazy;

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
        // TODO: instead of polling, use sender, receiver to tell us when we should call try_tick()
        thread::Builder::new()
            .name("child thread 1".to_string())
            .spawn(move || loop {
                get_task_pool().try_tick();
                thread::sleep(time::Duration::from_millis(sleep_millis));
            })
            .expect("unable to create child thread 1");
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
