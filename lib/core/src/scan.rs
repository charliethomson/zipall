use std::path::{Path, PathBuf};

use crate::{util::pathbuf_to_string, ZipAllError, ZipAllResult};

pub struct Scanner {
    source: PathBuf,
    dest: PathBuf,
}
impl Scanner {
    pub async fn new<P1: AsRef<Path>, P2: AsRef<Path>>(src: P1, dest: P2) -> ZipAllResult<Self> {
        Ok(Self {
            source: tokio::fs::canonicalize(src)
                .await
                .map_err(|e| ZipAllError::InvalidSource(e.to_string()))?,
            dest: tokio::fs::canonicalize(dest)
                .await
                .map_err(|e| ZipAllError::InvalidDestination(e.to_string()))?,
        })
    }
    pub async fn scan(&self) -> ZipAllResult<Vec<PathBuf>> {
        let mut paths = vec![];
        let src = pathbuf_to_string(&self.source)?;
        let dest = pathbuf_to_string(&self.dest)?;

        let mut iter = tokio::fs::read_dir(&self.source)
            .await
            .map_err(|e| ZipAllError::FailedToScanDirectory(src, e.to_string()))?;

        while let Some(entry) = iter
            .next_entry()
            .await
            .map_err(|e| ZipAllError::FailedToGetDirEntry(e.to_string()))?
        {
            let path = pathbuf_to_string(&entry.path())?;

            let meta = entry
                .metadata()
                .await
                .map_err(|e| ZipAllError::FailedToGetMetadata(e.to_string()))?;

            if !meta.is_dir() {
                log::trace!("Skipping non-directory {:?}", path);
                continue;
            }

            if path == dest {
                log::warn!("Skipping output directory: {:?}", dest);
                continue;
            }

            paths.push(entry.path());
        }

        Ok(paths)
    }
}
