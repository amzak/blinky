use std::fmt::{Display, Formatter};
use std::sync::PoisonError;

#[derive(Debug, Clone)]
pub struct Error(pub String);

impl<G> From<PoisonError<G>> for Error {
    fn from(_: PoisonError<G>) -> Self {
        Self("Concurrency error: the todo mutex has been poisoned".into())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error(format!("IO error: {error}"))
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error(error)
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error(String::from(error))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
