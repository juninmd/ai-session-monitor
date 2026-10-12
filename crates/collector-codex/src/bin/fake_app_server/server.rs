//! The conversation: one JSON message per line in, replies (and some noise) out.

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::{self, sleep};
use std::time::Duration;

use serde_json::{Value, json};

use crate::hold;
use crate::log::{self, Log};
use crate::mode::Mode;
use crate::replies::{self, Reply};

const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const NOT_INITIALIZED: i64 = -32002;
/// Per reply in `slow` mode: three of them overrun a short budget, one alone does not.
const SLOW_STEP: Duration = Duration::from_millis(600);
/// Longer than the 256 KiB line cap the client sets.
const HUGE_LINE: usize = 300_000;

pub fn run(mode: Mode, dir: &Path) -> io::Result<()> {
    log::write_file(dir, "fake-pid", &std::process::id().to_string());
    if let Ok(cwd) = std::env::current_dir() {
        log::write_file(dir, "fake-cwd", &cwd.to_string_lossy());
    }
    // A test that fails to clean up must not leave this running for long.
    thread::spawn(|| {
        sleep(hold::LIFETIME);
        std::process::exit(4);
    });
    if mode == Mode::ExitsEarly {
        return Ok(());
    }
    let holder = if mode == Mode::Grandchild { Some(start_holder(dir)?) } else { None };
    let mut server = Server {
        mode,
        out: io::stdout().lock(),
        log: Log::open(dir),
        saw_initialize: false,
        initialized: false,
    };
    let served = server.serve(io::stdin().lock());
    drop(holder);
    served
}

/// Shares stdout and stderr like the node process behind an npm shim: killing only the direct
/// child would leave the pipe open and the grandchild running.
fn start_holder(dir: &Path) -> io::Result<Child> {
    let child = Command::new(std::env::current_exe()?)
        .arg("hold")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    log::write_file(dir, "fake-child-pid", &child.id().to_string());
    Ok(child)
}

enum Flow {
    Continue,
    Stop,
}

struct Server {
    mode: Mode,
    out: io::StdoutLock<'static>,
    log: Log,
    saw_initialize: bool,
    initialized: bool,
}

impl Server {
    fn serve(&mut self, input: impl BufRead) -> io::Result<()> {
        for line in input.lines() {
            let line = line?;
            self.log.record(&line);
            let Ok(Value::Object(message)) = serde_json::from_str::<Value>(&line) else { continue };
            let method = message.get("method").and_then(Value::as_str);
            let id = message.get("id").filter(|id| !id.is_null());
            match (method, id) {
                (Some(method), Some(id)) => {
                    if matches!(self.request(method, id)?, Flow::Stop) {
                        return Ok(());
                    }
                }
                (Some("initialized"), None) => self.initialized = true,
                // Notifications and the client's answers to our own requests need no reply.
                _ => {}
            }
        }
        Ok(())
    }

    fn request(&mut self, method: &str, id: &Value) -> io::Result<Flow> {
        if self.mode == Mode::Hang {
            return Ok(Flow::Continue);
        }
        if self.mode == Mode::Noisy {
            self.chatter()?;
        }
        if self.mode == Mode::Slow {
            sleep(SLOW_STEP);
        }
        match method {
            "initialize" => self.initialize(id)?,
            // The real server insists on the whole handshake before anything else.
            _ if !self.initialized => self.fail(id, NOT_INITIALIZED, "Not initialized")?,
            "account/read" => self.answer(id, replies::account(self.mode))?,
            "account/rateLimits/read" => return self.rate_limits(id),
            _ => self.fail(id, METHOD_NOT_FOUND, "Method not found")?,
        }
        Ok(Flow::Continue)
    }

    fn initialize(&mut self, id: &Value) -> io::Result<()> {
        if self.saw_initialize {
            return self.fail(id, INVALID_REQUEST, "Already initialized");
        }
        self.saw_initialize = true;
        match self.mode {
            Mode::BadHandshake => self.fail(id, INVALID_REQUEST, "sentinel-handshake-text"),
            Mode::BadHandshakeResult => self.answer(id, Reply::Result(json!("not an object"))),
            Mode::GarbageLine => self.raw("this is not json sentinel-garbage"),
            Mode::HugeLine => {
                let pad = "x".repeat(HUGE_LINE);
                self.raw(&format!(r#"{{"id":{id},"result":{{"pad":"{pad}"}}}}"#))
            }
            _ => self.answer(id, Reply::Result(replies::initialize())),
        }
    }

    fn rate_limits(&mut self, id: &Value) -> io::Result<Flow> {
        match self.mode {
            Mode::HangAfterInit => {}
            Mode::ExitsMidway => return Ok(Flow::Stop),
            mode => self.answer(id, replies::rate_limits(mode))?,
        }
        Ok(Flow::Continue)
    }

    /// What a real server mixes in: a blank line, a notification carrying numbers that must never
    /// be mistaken for the answer, a reply to nobody, and a request of its own that the client
    /// has to refuse (the refusal shows up in the received log).
    fn chatter(&mut self) -> io::Result<()> {
        self.raw("")?;
        let update = json!({
            "method": "account/rateLimits/updated",
            "params": {"rateLimits": {"limitId": "codex", "primary": {
                "usedPercent": 99, "windowDurationMins": 300, "resetsAt": 1
            }}}
        });
        self.send(&update)?;
        self.send(&json!({"id": 999, "result": {}}))?;
        self.send(&json!({"id": "srv-1", "method": "item/tool/requestUserInput", "params": {}}))
    }

    fn answer(&mut self, id: &Value, reply: Reply) -> io::Result<()> {
        match reply {
            Reply::Result(result) => self.send(&json!({"id": id, "result": result})),
            Reply::Error { code, message } => self.fail(id, code, &message),
        }
    }

    fn fail(&mut self, id: &Value, code: i64, message: &str) -> io::Result<()> {
        self.send(&json!({"error": {"code": code, "message": message}, "id": id}))
    }

    fn send(&mut self, message: &Value) -> io::Result<()> {
        self.raw(&message.to_string())
    }

    fn raw(&mut self, line: &str) -> io::Result<()> {
        writeln!(self.out, "{line}")?;
        self.out.flush()
    }
}
