use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyErrors {
    #[error("Not implemented")]
    NotImplemented,

    #[error("Invalid operation. {0}")]
    InvalidOperation(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, MyErrors>;
