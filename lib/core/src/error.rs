use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ZipAllError {
    #[error("Failed to find local data directory.")]
    NoDataLocal,
    #[error("Failed to create directory: {0}")]
    FailedToCreateDirectory(String),
    #[error("IO: Failed to scan directory: {0}: {1}")]
    FailedToScanDirectory(String, String),
    #[error("Failed to configure logger: {0}")]
    Logger(String),
    #[error("Failed to convert OsStr to str")]
    FailedToConvertOsStr,
    #[error("Failed to get DirEntry: {0}")]
    FailedToGetDirEntry(String),
    #[error("Invalid regex: {0}")]
    InvalidRegex(String),
    #[error("Source directory is not present")]
    SourceDirectoryMissing,
    #[error("Failed to get metadata: {0}")]
    FailedToGetMetadata(String),
    #[error("Invalid Destination: {0}")]
    InvalidDestination(String),
    #[error("Invalid Source: {0}")]
    InvalidSource(String),
    #[error("Failed to read from file: {0}")]
    FailedToReadFromFile(String),
    #[error("Failed to write to file: {0}")]
    FailedToWriteToFile(String),
    #[error("Failed to send message over channel: {0}")]
    FailedToNotify(String),
    #[error("Failed to create archive: {0}: {1}")]
    FailedToCreateArchive(String, String),
    #[error("Failed to find source directory: {0}: {1}")]
    FailedToFindSource(String, String),
    #[error("Failed to spawn subprocess: {0}")]
    FailedToSpawn(String),
}
impl From<regex::Error> for ZipAllError {
    fn from(value: regex::Error) -> Self {
        Self::InvalidRegex(value.to_string())
    }
}

pub type ZipAllResult<T> = Result<T, ZipAllError>;
