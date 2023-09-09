use wasm_bindgen::prelude::*;

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
