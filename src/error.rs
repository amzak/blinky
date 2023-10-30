use std::sync::PoisonError;

#[derive(Debug)]
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