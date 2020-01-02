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

    pub fn read_s16(&mut self) -> Vec<Complex<f32>> {
        let mut resvec: Vec<Complex<f32>> = vec![];
        let mut buffer = [0u8; 1 << (13+1)];
        self.tcpstream.read_exact(&mut buffer).unwrap();
        for i in (0..buffer.len()).step_by(2*2) {
	    let mut num1: i32 = buffer[i+1] as i32 * 256 + buffer[i+0] as i32 ;
            let mut num2: i32 = buffer[i+3] as i32 * 256 + buffer[i+2] as i32 ;
            if num1 >= 0x8000 { num1 -= 0x10000 };
            if num2 >= 0x8000 { num2 -= 0x10000 };
            resvec.push(Complex::new(
                num1 as f32 / 64.,
                num2 as f32 / 64.,
            ));
        }
        resvec
    }

/*    pub fn read_f32(&mut self) -> Vec<Complex<f32>> {
        let mut resvec: Vec<Complex<f32>> = vec![];
        let mut buffer = [0u8; 1 << (13+2)];
        self.tcpstream.read_exact(&mut buffer).unwrap();
        let mut tmp1 = [0; 4];
        let mut tmp2 = [0; 4];
        for i in (0..buffer.len()).step_by(4*2) {
            tmp1.copy_from_slice(&buffer[i .. i+4]);
            tmp2.copy_from_slice(&buffer[i+4 .. i+8]);
            resvec.push(Complex::new(
                f32::from_ne_bytes(tmp1),
                f32::from_ne_bytes(tmp2)
            ));
        }
        resvec
    }
*/
    pub fn write_u8(&mut self, data: &[u8]) {
        self.tcpstream.write_all(&data).unwrap();
    }
}
