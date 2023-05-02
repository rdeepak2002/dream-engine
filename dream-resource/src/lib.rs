use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;

/// map to keep track of number of times a resource is being used, so the application can smartly deallocate it from the renderer and other consumers
pub(crate) static RESOURCE_MAP: Lazy<Mutex<HashMap<String, i32>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn using_resource(guid: String) {
    let new_count = RESOURCE_MAP.lock().unwrap().get(&*guid).unwrap_or(&0) + 1;
    println!(
        "[Resource Increase] Using resource {} {} times",
        guid, new_count
    );
    RESOURCE_MAP.lock().unwrap().insert(guid, new_count);
    // TODO: call this whenever an entity refers to a model / mesh of a model
    todo!();
}

pub fn no_longer_using_resource(guid: String) {
    // TODO: call this whenever an entity that refers to a model / mesh of a model is deleted
    let new_count = RESOURCE_MAP.lock().unwrap().get(&*guid).unwrap_or(&0) - 1;
    println!(
        "[Resource Decrease] Using resource {} {} times",
        guid, new_count
    );
    RESOURCE_MAP.lock().unwrap().insert(guid, new_count);
    // TODO: renderer should remove model if it is being used 0 times
    todo!()
}
