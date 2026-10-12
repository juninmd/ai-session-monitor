//! Splits a byte stream into UTF-8 lines under a length cap. Pure, so every edge is unit-tested.

#[derive(Debug, PartialEq, Eq)]
pub(super) enum LineError {
    /// A line grew past the cap; the stream cannot be trusted to stay aligned afterwards.
    TooLong,
    /// The line was consumed, so later lines are still aligned.
    NotUtf8,
}

pub(super) struct LineBuffer {
    pending: Vec<u8>,
    /// Prefix of `pending` already known to hold no newline, so each byte is scanned once.
    scanned: usize,
    max_line: usize,
}

impl LineBuffer {
    pub(super) fn new(max_line: usize) -> Self {
        Self { pending: Vec::new(), scanned: 0, max_line }
    }

    pub(super) fn push(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
    }

    /// The next complete line (without `\n` or a `\r` before it), if one is buffered. Without a
    /// newline yet, more than `max_line` bytes (plus a possible `\r`) is already too long, so the
    /// buffer never grows past the cap by more than one read.
    pub(super) fn next_line(&mut self) -> Result<Option<String>, LineError> {
        let Some(offset) = self.pending[self.scanned..].iter().position(|&b| b == b'\n') else {
            self.scanned = self.pending.len();
            if self.pending.len() > self.max_line.saturating_add(1) {
                return Err(self.discard());
            }
            return Ok(None);
        };
        let end = self.scanned + offset;
        let mut line: Vec<u8> = self.pending.drain(..=end).collect();
        self.scanned = 0;
        line.pop();
        self.decode(line).map(Some)
    }

    /// At end of stream: the unterminated last line, if the child left one.
    pub(super) fn finish(&mut self) -> Result<Option<String>, LineError> {
        self.scanned = 0;
        if self.pending.is_empty() {
            return Ok(None);
        }
        let line = std::mem::take(&mut self.pending);
        self.decode(line).map(Some)
    }

    fn decode(&mut self, mut line: Vec<u8>) -> Result<String, LineError> {
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        if line.len() > self.max_line {
            return Err(self.discard());
        }
        String::from_utf8(line).map_err(|_| LineError::NotUtf8)
    }

    fn discard(&mut self) -> LineError {
        self.pending = Vec::new();
        self.scanned = 0;
        LineError::TooLong
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(max: usize, chunks: &[&[u8]]) -> Vec<Result<Option<String>, LineError>> {
        let mut buffer = LineBuffer::new(max);
        let mut out = Vec::new();
        for chunk in chunks {
            buffer.push(chunk);
            loop {
                match buffer.next_line() {
                    Ok(None) => break,
                    Err(LineError::TooLong) => {
                        out.push(Err(LineError::TooLong));
                        break;
                    }
                    other => out.push(other),
                }
            }
        }
        out
    }

    fn some(text: &str) -> Result<Option<String>, LineError> {
        Ok(Some(text.to_owned()))
    }

    #[test]
    fn lines_split_across_reads_and_with_either_terminator() {
        let got = lines(64, &[b"al", b"pha\r\nbe", b"ta\n\ngam", b"ma\n"]);
        assert_eq!(got, [some("alpha"), some("beta"), some(""), some("gamma")]);
    }

    #[test]
    fn a_lone_carriage_return_is_content_not_a_terminator() {
        assert_eq!(lines(64, &[b"a\rb\n"]), [some("a\rb")]);
    }

    #[test]
    fn unicode_survives_being_cut_in_the_middle_of_a_character() {
        let bytes = "ação 🔑\n".as_bytes();
        let (head, tail) = bytes.split_at(2);
        assert_eq!(lines(64, &[head, tail]), [some("ação 🔑")]);
    }

    #[test]
    fn the_cap_counts_content_not_the_terminator() {
        assert_eq!(lines(5, &[b"12345\n"]), [some("12345")]);
        assert_eq!(lines(5, &[b"12345\r\n"]), [some("12345")]);
        assert_eq!(lines(5, &[b"123456\n"]), [Err(LineError::TooLong)]);
        assert_eq!(lines(5, &[b"123456\r\n"]), [Err(LineError::TooLong)]);
    }

    #[test]
    fn an_endless_line_is_cut_off_near_the_cap_without_waiting_for_a_newline() {
        let mut buffer = LineBuffer::new(16);
        buffer.push(&[b'x'; 17]);
        assert_eq!(buffer.next_line(), Ok(None), "16 bytes and a possible CR still fit");
        buffer.push(b"x");
        assert_eq!(buffer.next_line(), Err(LineError::TooLong));
        assert_eq!(buffer.pending.capacity(), 0, "the oversized data is released");
    }

    #[test]
    fn invalid_utf8_costs_one_line_and_keeps_the_stream_aligned() {
        let got = lines(64, &[b"ok\n\xFF\xFE\nnext\n"]);
        assert_eq!(got, [some("ok"), Err(LineError::NotUtf8), some("next")]);
    }

    #[test]
    fn the_last_unterminated_line_is_returned_once_at_end_of_stream() {
        let mut buffer = LineBuffer::new(64);
        buffer.push(b"done\r\npartial");
        assert_eq!(buffer.next_line(), Ok(Some("done".into())));
        assert_eq!(buffer.next_line(), Ok(None));
        assert_eq!(buffer.finish(), Ok(Some("partial".into())));
        assert_eq!(buffer.finish(), Ok(None));
    }

    #[test]
    fn a_zero_cap_allows_only_empty_lines() {
        assert_eq!(lines(0, &[b"\n\r\n"]), [some(""), some("")]);
        assert_eq!(lines(0, &[b"x\n"]), [Err(LineError::TooLong)]);
    }

    #[test]
    fn a_huge_cap_does_not_overflow() {
        assert_eq!(lines(usize::MAX, &[b"a\n"]), [some("a")]);
    }
}
