use std::future::Future;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{thread, time};

use async_executor::Executor;
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
    pub fn init(&self, sleep_millis: u64) {
        thread::spawn(move || loop {
            get_task_pool().try_tick();
            thread::sleep(time::Duration::from_millis(sleep_millis));
        });
    }

    pub fn try_tick(&self) {
        if !self.executor.is_empty() {
            self.executor.try_tick();
        }
    }

    pub fn spawn<T: Send + 'task>(&self, future: impl Future<Output = T> + Send + 'task) {
        let _task = self.executor.spawn(async move { future.await });
        _task.detach();
    }
}
