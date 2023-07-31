use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ResourceHandle {
    pub key: String,
    pub path: PathBuf,
}

impl ResourceHandle {
    pub fn new(key: String, path: PathBuf) -> Self {
        Self { key, path }
    }
}
