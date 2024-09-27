use core::{data_base_dir, join_create_dir, join_create_file};
use std::path::PathBuf;
use zipall_core::ZipAllResult;

mod core {
    use std::path::{Path, PathBuf};

    use zipall_core::{ZipAllError, ZipAllResult, PRODUCT_NAME};

    pub fn data_base_dir() -> ZipAllResult<PathBuf> {
        Ok(dirs::data_local_dir()
            .ok_or(ZipAllError::NoDataLocal)?
            .join(PRODUCT_NAME))
    }

    pub fn join_create_dir<P: AsRef<Path>>(base: PathBuf, ext: P) -> ZipAllResult<PathBuf> {
        if !base.exists() {
            std::fs::create_dir_all(&base)
                .map_err(|e| ZipAllError::FailedToCreateDirectory(e.to_string()))?;
        }

        let dir = base.join(ext);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)
                .map_err(|e| ZipAllError::FailedToCreateDirectory(e.to_string()))?;
        }

        Ok(dir)
    }

    pub fn join_create_file<P: AsRef<Path>>(base: PathBuf, filename: P) -> ZipAllResult<PathBuf> {
        if !base.exists() {
            std::fs::create_dir_all(&base)
                .map_err(|e| ZipAllError::FailedToCreateDirectory(e.to_string()))?;
        }
        Ok(base.join(filename))
    }
}

fn logs_base_dir() -> ZipAllResult<PathBuf> {
    join_create_dir(data_base_dir()?, "logs")
}

fn logs_dir(module_name: &str) -> ZipAllResult<PathBuf> {
    let parts = module_name.split("::");
    let mut dir = logs_base_dir()?;
    for part in parts {
        dir = join_create_dir(dir, part)?;
    }
    Ok(dir)
}
pub fn log_file(module_name: &str) -> ZipAllResult<PathBuf> {
    let slug = module_name.replace("::", "_");
    join_create_file(logs_dir(module_name)?, slug)
}
