use std::time::{Duration, SystemTime, UNIX_EPOCH};

use wasm_bindgen::prelude::*;

pub fn now() -> u128 {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let window = web_sys::window().expect("should have a window in this context");
            let performance = window
                .performance()
                .expect("performance should be available");
            performance.now() as u128
        } else {
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            since_the_epoch.as_millis()
        }
    }
}
