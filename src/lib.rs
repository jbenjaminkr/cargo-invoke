use anyhow::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub fn initialize() -> Result<()> {
    Ok(())
}
