use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Fail to open file: {0}")]
    FailToOpenFile(PathBuf),
}