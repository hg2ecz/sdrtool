use std::io::{self, BufRead};
use std::sync::mpsc;
use std::thread;
use std::time;

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
        CmdIn { rx }
    }

    // f-0.3  --> set mixfreq to -0.3 MHz
    pub fn get(&self) -> Option<(char, f64)> {
        let d = time::Duration::from_millis(0);
        if let Ok(val) = self.rx.recv_timeout(d) {
            let s_in = val.unwrap(); // get...
            let s_in = s_in.trim_end();
            if s_in.len() > 1 {
                let c: char = s_in.chars().next().unwrap();
                let s: &str = &s_in[1..];
                if let Ok(sf64) = s.parse() {
                    return Some((c, sf64));
                }
            }
        }
        None // if not Some(...)
    }
}

impl Default for CmdIn {
    fn default() -> Self {
        Self::new()
    }
}
