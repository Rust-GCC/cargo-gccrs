mod args;
mod config;
mod env_args;
mod error;
mod gccrs;
mod rustc_args;

pub use error::Error;
pub use gccrs::Gccrs;

pub type Result<T = ()> = std::result::Result<T, Error>;
