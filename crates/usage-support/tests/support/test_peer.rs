//! Test-only peer process for the `proc` tests: Cargo hands its path to integration tests, which
//! pick one behaviour through the first argument. It prints through `Write`, not `println!`,
//! because the output is the protocol.

use std::io::{self, BufRead, Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::thread::sleep;
use std::time::Duration;

const BLOCK: usize = 8 * 1024;

fn number(args: &[String], index: usize) -> u64 {
    args.get(index).and_then(|value| value.parse().ok()).unwrap_or(0)
}

fn pause(millis: u64) {
    sleep(Duration::from_millis(millis));
}

/// Writes `count` bytes of `x` in blocks, then `tail`.
fn write_bytes(mut out: impl Write, count: u64, tail: &[u8]) -> io::Result<()> {
    let block = [b'x'; BLOCK];
    let mut left = count;
    while left > 0 {
        let take = usize::try_from(left).map_or(BLOCK, |n| n.min(BLOCK));
        out.write_all(&block[..take])?;
        left -= take as u64;
    }
    out.write_all(tail)?;
    out.flush()
}

/// One reply per input line, flushed at once, until stdin closes.
fn echo() -> io::Result<()> {
    let mut out = io::stdout().lock();
    for line in io::stdin().lock().lines() {
        writeln!(out, "got:{}", line?)?;
        out.flush()?;
    }
    Ok(())
}

fn cat() -> io::Result<()> {
    let mut input = io::stdin().lock();
    let mut out = io::stdout().lock();
    let mut block = vec![0u8; 64 * 1024];
    loop {
        let read = input.read(&mut block)?;
        if read == 0 {
            return out.flush();
        }
        out.write_all(&block[..read])?;
    }
}

/// Writes until the reader goes away.
fn flood() -> io::Result<()> {
    let mut out = io::stdout().lock();
    let block = [b'x'; BLOCK];
    loop {
        out.write_all(&block)?;
    }
}

fn print_args(args: &[String]) -> io::Result<()> {
    let mut out = io::stdout().lock();
    for arg in args {
        writeln!(out, "[{arg}]")?;
    }
    out.flush()
}

fn print_env(name: &str) -> io::Result<()> {
    let value = std::env::var(name).unwrap_or_else(|_| "<unset>".to_owned());
    let mut out = io::stdout().lock();
    writeln!(out, "{value}")?;
    out.flush()
}

fn print_cwd() -> io::Result<()> {
    let dir = std::env::current_dir()?;
    let mut out = io::stdout().lock();
    writeln!(out, "{}", dir.display())?;
    out.flush()
}

/// Runs `resolve_binary` from this process's working directory, which the test chooses, to show
/// that the directory is never searched.
fn resolve(args: &[String]) -> io::Result<()> {
    let name = args.first().map_or("", String::as_str);
    let path_env = args.get(1).map_or("", String::as_str);
    let found = usage_support::resolve_binary(name, path_env, ".COM;.EXE;.BAT;.CMD", &[]);
    let mut out = io::stdout().lock();
    match found {
        Some(path) => writeln!(out, "{}", path.display())?,
        None => writeln!(out, "none")?,
    }
    out.flush()
}

/// Starts a copy of itself that sleeps and reports its pid on stdout (and in `pid_file`, for
/// callers that never see stdout). `hold_pipes` lets the copy inherit stdout so it keeps the
/// pipe open after this process is gone.
fn spawn_sleeper(hold_pipes: bool, millis: u64, pid_file: Option<&String>) -> io::Result<()> {
    let stdio = || if hold_pipes { Stdio::inherit() } else { Stdio::null() };
    let child = Command::new(std::env::current_exe()?)
        .args(["sleep", &millis.to_string()])
        .stdin(Stdio::null())
        .stdout(stdio())
        .stderr(stdio())
        .spawn()?;
    if let Some(file) = pid_file {
        std::fs::write(file, child.id().to_string())?;
    }
    let mut out = io::stdout().lock();
    writeln!(out, "child:{}", child.id())?;
    out.flush()
}

fn run(args: &[String]) -> io::Result<()> {
    match args.first().map(String::as_str) {
        Some("echo") => echo(),
        Some("cat") => cat(),
        Some("flood") => flood(),
        Some("args") => print_args(&args[1..]),
        Some("cwd") => print_cwd(),
        Some("env") => print_env(args.get(1).map_or("", String::as_str)),
        Some("resolve") => resolve(&args[1..]),
        Some("print") => write_bytes(io::stdout().lock(), number(args, 1), b""),
        Some("print-err") => write_bytes(io::stderr().lock(), number(args, 1), b""),
        Some("line") => write_bytes(io::stdout().lock(), number(args, 1), b"\n"),
        Some("print-both") => {
            write_bytes(io::stderr().lock(), number(args, 2), b"")?;
            write_bytes(io::stdout().lock(), number(args, 1), b"")
        }
        Some("sleep") => {
            pause(number(args, 1));
            Ok(())
        }
        Some("partial") => {
            let mut out = io::stdout().lock();
            out.write_all(b"part")?;
            out.flush()?;
            pause(number(args, 1));
            out.write_all(b"ial\n")?;
            out.flush()
        }
        Some("grandchild") => {
            spawn_sleeper(false, 30_000, args.get(1))?;
            pause(30_000);
            Ok(())
        }
        Some("grandchild-until-eof") => {
            spawn_sleeper(false, 30_000, args.get(1))?;
            // Leaves as soon as its stdin closes, the way a shim stops when its parent hangs up.
            io::copy(&mut io::stdin().lock(), &mut io::sink()).map(|_| ())
        }
        Some("orphan-holding-pipe") => spawn_sleeper(true, number(args, 1), None),
        _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("exit") {
        return ExitCode::from(u8::try_from(number(&args, 1)).unwrap_or(255));
    }
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        // A reader that left early (broken pipe) is how several tests end the peer.
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(64),
    }
}
