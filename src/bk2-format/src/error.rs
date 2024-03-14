use thiserror::Error;

#[derive(Error, Debug)]
pub enum Bk2Error {
    #[error("IO error: {0}")]
    IoError(std::io::Error),

    #[error("Zip error: {0}")]
    ZipError(zip::result::ZipError),
}
