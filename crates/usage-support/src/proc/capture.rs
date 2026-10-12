//! Bounded I/O with a child's pipes: keep the head of the output, drop the rest, and never let
//! a full pipe stall the child.

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const CHUNK_BYTES: usize = 8 * 1024;
/// Largest buffer reserved up front; the cap is only a limit, not a promise of that much output.
const PREALLOCATE_LIMIT: usize = 64 * 1024;

pub(super) struct Capture {
    pub(super) bytes: Vec<u8>,
    pub(super) truncated: bool,
    cap: usize,
}

impl Capture {
    pub(super) fn new(cap: usize) -> Self {
        Self { bytes: Vec::with_capacity(cap.min(PREALLOCATE_LIMIT)), truncated: false, cap }
    }

    /// Reads to end of stream. Bytes past the cap are read and discarded, so a chatty child keeps
    /// running instead of blocking on a full pipe, and memory stays at the cap.
    pub(super) async fn fill<R: AsyncRead + Unpin>(&mut self, mut reader: R) {
        let mut chunk = [0u8; CHUNK_BYTES];
        loop {
            match reader.read(&mut chunk).await {
                Ok(0) | Err(_) => return,
                Ok(read) => {
                    let keep = read.min(self.cap - self.bytes.len());
                    self.bytes.extend_from_slice(&chunk[..keep]);
                    self.truncated |= keep < read;
                }
            }
        }
    }
}

/// Writes `bytes` and closes the pipe. A child that exits without reading everything breaks the
/// pipe; that is its right and not an error.
pub(super) async fn feed<W: AsyncWrite + Unpin>(writer: Option<W>, bytes: Option<Vec<u8>>) {
    if let (Some(mut writer), Some(bytes)) = (writer, bytes) {
        let _ = writer.write_all(&bytes).await;
        let _ = writer.shutdown().await;
    }
}
