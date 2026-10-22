//! One connection: read a bounded head before a deadline, answer once, close. No keep-alive, no
//! bodies, no pipelining, and nothing of the request is logged.

use std::io::{self, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::{Duration, Instant};

use super::request::{MAX_HEAD_BYTES, Malformed, parse};
use super::response::{Response, route};
use super::server::{Limits, Provider};

enum ReadError {
    TooLarge,
    TimedOut,
    /// The peer left or the socket failed: there is nobody to answer.
    Gone,
}

pub(super) fn serve(mut stream: TcpStream, port: u16, provider: &Provider, limits: Limits) {
    let response = match read_head(&mut stream, Instant::now() + limits.read_deadline) {
        Ok(head) => match parse(&head) {
            Ok(request) => route(&request, port, provider),
            Err(Malformed::BadRequest) => Response::error(400),
            Err(Malformed::TooLarge) => Response::error(431),
        },
        Err(ReadError::TooLarge) => Response::error(431),
        Err(ReadError::TimedOut) => Response::error(408),
        Err(ReadError::Gone) => return,
    };
    if stream.set_write_timeout(Some(limits.write_timeout)).is_err() {
        return;
    }
    if stream.write_all(&response.to_bytes()).is_ok() {
        let _ = stream.flush();
    }
    let _ = stream.shutdown(Shutdown::Write);
    drain(&mut stream);
}

/// Reads up to the blank line. The deadline covers the whole head, so a client that trickles a
/// byte at a time is cut off like one that says nothing.
fn read_head(stream: &mut TcpStream, deadline: Instant) -> Result<Vec<u8>, ReadError> {
    let mut head = Vec::with_capacity(512);
    let mut chunk = [0_u8; 1024];
    loop {
        let left = deadline
            .checked_duration_since(Instant::now())
            .filter(|left| !left.is_zero())
            .ok_or(ReadError::TimedOut)?;
        stream.set_read_timeout(Some(left)).map_err(|_| ReadError::Gone)?;
        match stream.read(&mut chunk) {
            Ok(0) => return Err(ReadError::Gone),
            Ok(read) => head.extend_from_slice(&chunk[..read]),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err)
                if matches!(err.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) =>
            {
                return Err(ReadError::TimedOut);
            }
            Err(_) => return Err(ReadError::Gone),
        }
        if let Some(end) = head.windows(4).position(|w| w == b"\r\n\r\n") {
            head.truncate(end + 4);
            return if head.len() > MAX_HEAD_BYTES { Err(ReadError::TooLarge) } else { Ok(head) };
        }
        if head.len() > MAX_HEAD_BYTES {
            return Err(ReadError::TooLarge);
        }
    }
}

/// Closing a socket that still holds unread input makes the OS reset it, which can discard the
/// response in flight. Reading a little, briefly, lets a refused request see its answer.
fn drain(stream: &mut TcpStream) {
    let deadline = Instant::now() + Duration::from_millis(250);
    let mut sink = [0_u8; 4096];
    let mut total = 0_usize;
    while total < 64 * 1024 {
        let Some(left) = deadline.checked_duration_since(Instant::now()).filter(|d| !d.is_zero())
        else {
            return;
        };
        if stream.set_read_timeout(Some(left)).is_err() {
            return;
        }
        match stream.read(&mut sink) {
            Ok(0) | Err(_) => return,
            Ok(read) => total += read,
        }
    }
}
