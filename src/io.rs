#![allow(dead_code)]
use num_complex::Complex;
use std::io::{self, Read, Write};
use std::net::{TcpStream};

pub fn read_stdin_u8() -> Vec<Complex<f32>> {
    let mut buffer = [0u8; 1<<13];
    let mut resvec: Vec<Complex<f32>> = vec![];
    io::stdin().read(&mut buffer).unwrap();
    for i in (0..buffer.len()).step_by(2) {
        resvec.push( Complex::new(buffer[i] as f32, buffer[i+1] as f32) );
    }
    return resvec;
}

pub fn write_stdout_i16(soundout: Vec<i16>) {
    let mut outbytes = vec![];
    for x in soundout {
        outbytes.push((x & 0xff) as u8);
        outbytes.push((x >> 8) as u8);
    }
    io::stdout().write_all(&outbytes).unwrap();
}

// Parameter: "ipaddress:port"
pub struct Sdrtcpcli {
    tcpstream: TcpStream,
}

impl Sdrtcpcli {
    pub fn new(ip_port: &str) -> Sdrtcpcli {
        Sdrtcpcli { tcpstream: TcpStream::connect(ip_port).unwrap() }
    }

    pub fn get(&mut self) -> Vec<Complex<f32>> {
        let mut resvec: Vec<Complex<f32>> = vec![];
        let mut buffer = [0u8; 1<<13];
        self.tcpstream.read(&mut buffer).unwrap();
        for i in (0..buffer.len()).step_by(2) {
            resvec.push( Complex::new(buffer[i] as f32, buffer[i+1] as f32) );
        }
        return resvec;
    }
}
