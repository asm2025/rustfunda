pub mod auth;
pub mod datetime;
pub mod error;
pub mod io;
pub mod threading;

pub type Result<T> = std::result::Result<T, error::RmxError>;
