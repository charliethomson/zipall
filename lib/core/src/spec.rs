use std::path::{Path, PathBuf};

use crate::{ZipAllError, ZipAllResult};

#[derive(Debug, Clone)]
pub enum ZipMode {
    SevenZed,
}
impl ZipMode {
    fn add_extension(&self, filename: &str) -> String {
        let ext = match self {
            ZipMode::SevenZed => "7z",
        };

        format!("{}.{}", filename, ext)
    }
}

#[derive(Debug, Clone)]
pub struct ZipSpecification {
    pub filename: String,
    pub source: PathBuf,
    pub dest: PathBuf,
    pub mode: ZipMode,
}
impl ZipSpecification {
    pub fn new<P1: AsRef<Path>, P2: AsRef<Path>>(
        source: P1,
        dest_base: P2,
        mode: ZipMode,
    ) -> ZipAllResult<Self> {
        let source = PathBuf::from(source.as_ref());
        let filename = source
            .file_name()
            .ok_or(ZipAllError::FailedToConvertOsStr)?;
        let filename = filename.to_str().ok_or(ZipAllError::FailedToConvertOsStr)?;
        let dest = PathBuf::from(dest_base.as_ref()).join(mode.add_extension(filename));

        Ok(Self {
            filename: filename.to_string(),
            source,
            dest,
            mode,
        })
    }
}
