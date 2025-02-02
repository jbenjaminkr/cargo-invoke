pub mod crate::common;
pub mod crate::config;
pub mod crate::error;

pub use config::Config;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub fn initialize() -> Result<()> {
    Ok(())
}
