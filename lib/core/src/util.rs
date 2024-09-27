use std::path::Path;

use crate::{ZipAllError, ZipAllResult};

pub fn pathbuf_to_string(path: &Path) -> ZipAllResult<String> {
    let os = path.as_os_str();
    let str = os.to_str().ok_or(ZipAllError::FailedToConvertOsStr)?;
    Ok(str.to_string())
}
