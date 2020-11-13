use std::io::{self, BufRead};
use std::thread;
use std::time;
use std::sync::mpsc;

pub struct CmdIn {
    rx: mpsc::Receiver<Result<String, io::Error>>,
}

impl CmdIn {
    pub fn new() -> CmdIn {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let input = io::stdin();
            for line in input.lock().lines() {
                tx.send(line).unwrap();
            }
        });
        return CmdIn {rx: rx};
    }

    // f-0.3  --> set mixfreq to -0.3 MHz
    pub fn getstring(&self) -> Option<String> {
        let d = time::Duration::from_millis(0);
        if let Ok(val) = self.rx.recv_timeout(d) {
            Some(val.unwrap())
        } else {
            None
        }
    }
}
