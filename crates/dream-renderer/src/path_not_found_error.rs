#[derive(Debug, Clone)]
pub struct PathNotFoundError;

impl std::fmt::Display for PathNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "unable to find file at path")
    }
}
