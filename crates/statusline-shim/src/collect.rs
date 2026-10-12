//! Reads a child's stdout on a helper thread into a shared buffer, so the caller can judge the
//! command by its exit instead of by its pipe closing. A background job the command left running
//! inherits that pipe and keeps it open, but everything the command printed is already buffered.

use std::io::{self, Read, Write};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::Duration;

pub struct Collector {
    bytes: Arc<Mutex<Vec<u8>>>,
    /// Signalled (or disconnected) when the reader stops: end of input, the cap, or a read error.
    stopped: Receiver<()>,
    limit: usize,
}

impl Collector {
    /// Starts reading `source`. At most `limit + 1` bytes are kept and nothing past them is read:
    /// the extra byte only marks output that is too long.
    pub fn start(source: impl Read + Send + 'static, limit: usize) -> Self {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let (stop, stopped) = mpsc::channel();
        let mut sink = Sink(Arc::clone(&bytes));
        let budget = u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1);
        thread::spawn(move || {
            // A read error ends the reading like end of input: what arrived before it is kept.
            let _ = io::copy(&mut source.take(budget), &mut sink);
            let _ = stop.send(());
        });
        Self { bytes, stopped, limit }
    }

    /// Lets the caller end a command that keeps printing instead of waiting for it to stop.
    pub fn over_limit(&self) -> bool {
        locked(&self.bytes).len() > self.limit
    }

    /// Waits up to `window` for the source to end, then hands back what has been read so far,
    /// or nothing when that is more than `limit` bytes.
    pub fn finish(self, window: Duration) -> Option<Vec<u8>> {
        let _ = self.stopped.recv_timeout(window);
        let bytes = std::mem::take(&mut *locked(&self.bytes));
        (bytes.len() <= self.limit).then_some(bytes)
    }
}

/// Appends to the shared buffer, so the bytes read so far are visible while the reader blocks.
struct Sink(Arc<Mutex<Vec<u8>>>);

impl Write for Sink {
    fn write(&mut self, chunk: &[u8]) -> io::Result<usize> {
        locked(&self.0).extend_from_slice(chunk);
        Ok(chunk.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A poisoned lock only means a holder panicked; a byte buffer has no invariant to break.
fn locked(bytes: &Mutex<Vec<u8>>) -> MutexGuard<'_, Vec<u8>> {
    bytes.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    /// Far longer than any test should take: a test that waits it out has failed.
    const LONG: Duration = Duration::from_secs(30);

    /// Polls state rather than guessing how long a thread needs; the cap only stops a hang.
    fn wait_until(mut ready: impl FnMut() -> bool) {
        let give_up = Instant::now() + Duration::from_secs(20);
        while !ready() {
            assert!(Instant::now() < give_up, "the reader never caught up");
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn buffered(collector: &Collector) -> usize {
        locked(&collector.bytes).len()
    }

    #[test]
    fn a_source_that_has_ended_is_returned_at_once_however_long_the_window() {
        let (reader, mut writer) = io::pipe().unwrap();
        let collector = Collector::start(reader, 1024);
        writer.write_all("olá ✓".as_bytes()).unwrap();
        drop(writer);

        let started = Instant::now();
        let bytes = collector.finish(LONG);

        assert_eq!(bytes.as_deref(), Some("olá ✓".as_bytes()));
        assert!(started.elapsed() < LONG / 3, "waited out the window: {:?}", started.elapsed());
    }

    #[test]
    fn a_source_that_stays_open_costs_one_window_and_keeps_what_was_read() {
        let (reader, mut writer) = io::pipe().unwrap();
        let collector = Collector::start(reader, 1024);
        writer.write_all(b"line").unwrap();
        wait_until(|| buffered(&collector) == 4);

        // The writer is still alive, as a background job holding the pipe would be.
        let bytes = collector.finish(Duration::from_millis(50));

        assert_eq!(bytes.as_deref(), Some(b"line".as_slice()));
        drop(writer);
    }

    #[test]
    fn an_empty_source_gives_empty_output() {
        let (reader, writer) = io::pipe().unwrap();
        let collector = Collector::start(reader, 1024);
        drop(writer);

        assert!(!collector.over_limit());
        assert_eq!(collector.finish(LONG), Some(Vec::new()));
    }

    #[test]
    fn exactly_the_limit_is_not_over_it() {
        let (reader, mut writer) = io::pipe().unwrap();
        let collector = Collector::start(reader, 8);
        writer.write_all(&[b'x'; 8]).unwrap();
        drop(writer);
        wait_until(|| buffered(&collector) == 8);

        assert!(!collector.over_limit());
        assert_eq!(collector.finish(LONG).map(|bytes| bytes.len()), Some(8));
    }

    #[test]
    fn one_byte_past_the_limit_is_over_it_and_nothing_more_is_kept() {
        let (reader, mut writer) = io::pipe().unwrap();
        let collector = Collector::start(reader, 8);
        // Fits the pipe buffer; the writer stays open, so only the cap can stop the reader.
        writer.write_all(&[b'x'; 100]).unwrap();

        wait_until(|| collector.over_limit());

        assert_eq!(
            buffered(&collector),
            9,
            "limit + 1 marks the overflow; nothing is read past it"
        );
        assert_eq!(collector.finish(LONG), None);
        drop(writer);
    }

    #[test]
    fn output_past_the_limit_is_refused_even_if_nobody_checked_before_the_end() {
        let (reader, mut writer) = io::pipe().unwrap();
        let collector = Collector::start(reader, 8);
        writer.write_all(&[b'x'; 9]).unwrap();
        drop(writer);

        assert_eq!(collector.finish(LONG), None);
    }

    struct FailsAfter(&'static [u8]);

    impl Read for FailsAfter {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if self.0.is_empty() {
                return Err(io::Error::other("the pipe broke"));
            }
            let n = self.0.len().min(buf.len());
            buf[..n].copy_from_slice(&self.0[..n]);
            self.0 = &self.0[n..];
            Ok(n)
        }
    }

    #[test]
    fn a_read_error_keeps_what_arrived_before_it() {
        let collector = Collector::start(FailsAfter(b"partial"), 1024);

        assert_eq!(collector.finish(LONG).as_deref(), Some(b"partial".as_slice()));
    }
}
