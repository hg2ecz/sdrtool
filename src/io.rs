#![allow(dead_code)]
use num_complex::Complex;
use std::io::{self, Read, Write};
use std::net::TcpStream;

/// ```read [u8; 8192] from stdin and convert to Vec<Complex<f32>>```
pub fn read_stdin_u8() -> Vec<Complex<f32>> {
    let mut buffer = [0u8; 1 << 13];
    let mut resvec: Vec<Complex<f32>> = vec![];
    io::stdin().read_exact(&mut buffer).unwrap();
    for i in (0..buffer.len()).step_by(2) {
        resvec.push(Complex::new(
            f32::from(buffer[i]) - f32::from(std::u8::MAX) / 2.,
            f32::from(buffer[i + 1]) - f32::from(std::u8::MAX) / 2.,
        ));
    }
    resvec
}

/// ```convert &[f32] to Vec<i16> and write to stdout```
pub fn write_stdout_i16(soundout: &[f32]) {
    let mut outbytes = vec![];
    for x in soundout {
        let xi = *x as i16;
        outbytes.push((xi & 0xff) as u8);
        outbytes.push((xi >> 8) as u8);
    }
    io::stdout().write_all(&outbytes).unwrap();
}

// Parameter: "ipaddress:port"
/// ```TCP client (read_u8, write_u8)```
pub struct Sdrtcpcli {
    tcpstream: TcpStream,
}

impl Sdrtcpcli {
    pub fn new(ip_port: &str) -> Sdrtcpcli {
        Sdrtcpcli {
            tcpstream: TcpStream::connect(ip_port).unwrap(),
        }
    }

    pub fn read_u8(&mut self) -> Vec<Complex<f32>> {
        let mut resvec: Vec<Complex<f32>> = vec![];
        let mut buffer = [0u8; 1 << 13];
        self.tcpstream.read_exact(&mut buffer).unwrap();
        for i in (0..buffer.len()).step_by(2) {
            resvec.push(Complex::new(
                f32::from(buffer[i]) - f32::from(std::u8::MAX) / 2.,
                f32::from(buffer[i + 1]) - f32::from(std::u8::MAX) / 2.,
            ));
        }
        resvec
    }

    pub fn write_u8(&mut self, data: &[u8]) {
        self.tcpstream.write_all(&data).unwrap();
    }
}
