use std::{
    borrow::Cow,
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use humansize::DECIMAL;
use lazy_static::lazy_static;
use lzma::LzmaWriter;
use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWrite, BufReader},
    sync::mpsc::UnboundedSender,
};

use crate::{util::pathbuf_to_string, ZipAllError, ZipAllResult, ZipSpecification};

lazy_static! {
    static ref RE_PERCENT: Regex = Regex::new(r#"(\d{1,3})%"#).unwrap();
}

#[derive(Clone)]
pub enum ZipStat {
    Progress {
        id: usize,
        filename: String,
        percent: usize,
    },
    KeepAlive(usize),
}

pub struct Zipper {
    id: usize,
    spec: ZipSpecification,
    filename: String,
    bin: PathBuf,
    tx: UnboundedSender<ZipStat>,
}
impl Zipper {
    pub fn new(
        spec: ZipSpecification,
        id: usize,
        bin: PathBuf,
        tx: UnboundedSender<ZipStat>,
    ) -> Self {
        Self {
            id,
            filename: spec.filename.clone(),
            spec,
            bin,
            tx,
        }
    }

    pub async fn run(self) -> ZipAllResult<usize> {
        let mut command = tokio::process::Command::new(pathbuf_to_string(&self.bin)?);
        command
            .arg("a")
            .arg(pathbuf_to_string(&self.spec.dest)?)
            .arg(pathbuf_to_string(&self.spec.source)?);
        command
            .arg("-bso0") // stdout >> /dev/stdin (dont care :3)
            .arg("-bse2") // stderr >> /dev/stderr
            .arg("-bsp1"); // progress >> /dev/stdout

        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command
            .spawn()
            .map_err(|e| ZipAllError::FailedToSpawn(e.to_string()))?;

        let stdout = child.stdout.take().expect("7z should have a stdout");
        let mut out_reader = BufReader::new(stdout);
        let mut out_buffer = Vec::with_capacity(100);
        let stderr = child.stderr.take().expect("7z should have a stderr");
        let mut err_reader = BufReader::new(stderr);
        let mut err_buffer = Vec::with_capacity(100);

        let mut interval = tokio::time::interval(Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.tx.send(ZipStat::KeepAlive(self.id)).expect("Failed to send message");
                },
                Ok(_) = out_reader.read_until(b'%', &mut out_buffer) => {
                    let line = String::from_utf8_lossy(&out_buffer);
                    if line.is_empty() { continue; }

                    let Some(percent_str) = RE_PERCENT
                        .captures(&line)
                        .and_then(|cap| cap.get(1))
                        .map(|cap| cap.as_str())
                    else {
                        log::warn!(
                            "[{}] Failed to find percent in progress line! [1]: \"{line}\"",
                            self.id
                        );
                        out_buffer.clear();
                        continue;
                    };

                    let Ok(percent) = percent_str.parse::<usize>() else {
                        log::warn!("[{}] Failed to parse progress line! [1]: \"{line}\"", self.id);
                        out_buffer.clear();
                        continue;
                    };

                    self.tx.send(ZipStat::Progress { id: self.id, filename: self.filename.clone(),  percent }).expect("Failed to send message");
                    out_buffer.clear();
                },
                Ok(_) = err_reader.read_until(b'%', &mut err_buffer) => {
                    let line = String::from_utf8_lossy(&err_buffer);
                    if line.is_empty() { continue; }
                    log::warn!("[STDERR] \"{line}\"");
                    err_buffer.clear();
                },
                result = child.wait() => {
                    log::info!("[{}] Exited with {:?}", self.id, result);
                    break;
                },
            }
        }

        Ok(self.id)
    }
}
