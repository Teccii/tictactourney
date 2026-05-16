use crate::ugi::UgiCommand;
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{Receiver, channel},
    time::{Duration, Instant},
};

#[derive(Debug)]
pub struct Engine {
    _child: Child,
    stdin: ChildStdin,
    rx: Receiver<String>,
}

impl Engine {
    #[inline]
    pub fn new(path: &str) -> Self {
        let mut child = Command::new(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();

        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if tx.send(line).is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        Engine {
            _child: child,
            stdin,
            rx,
        }
    }

    #[inline]
    pub fn ping(&mut self) {
        self.send(UgiCommand::IsReady);
        self.wait_for(Duration::from_millis(1000), |str| {
            str.starts_with("readyok")
        });
    }

    #[inline]
    pub fn send(&mut self, cmd: UgiCommand) {
        writeln!(self.stdin, "{cmd}").unwrap();
        self.stdin.flush().unwrap();
    }

    #[inline]
    pub fn read_until(
        &mut self,
        timeout: Duration,
        pred: impl Fn(&str) -> bool,
    ) -> Option<(Vec<String>, String)> {
        let start = Instant::now();
        let mut lines = Vec::new();

        loop {
            let remaining = timeout.checked_sub(start.elapsed()).unwrap();
            let line = self.rx.recv_timeout(remaining).ok()?;

            if pred(&line) {
                return Some((lines, line));
            } else {
                lines.push(line);
            }
        }
    }

    #[inline]
    pub fn wait_for(&mut self, timeout: Duration, pred: impl Fn(&str) -> bool) -> Option<String> {
        let start = Instant::now();
        loop {
            let remaining = timeout.checked_sub(start.elapsed()).unwrap();
            let line = self.rx.recv_timeout(remaining).ok()?;

            if pred(&line) {
                return Some(line);
            }
        }
    }
}
