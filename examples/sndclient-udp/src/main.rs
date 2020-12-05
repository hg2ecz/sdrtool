use std::net;

use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};

mod command_in;

fn sound_init(pcm: &PCM, samplerate: u32, channelnum: u32) -> alsa::pcm::IO<i16> {
    // Set hardware parameters: 48000 Hz / Mono / 16 bit
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(channelnum).unwrap();
    hwp.set_rate(samplerate, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    pcm.hw_params(&hwp).unwrap();
    let io = pcm.io_i16().unwrap();

    // Make sure we don't start the stream too early
    let hwp = pcm.hw_params_current().unwrap();
    let swp = pcm.sw_params_current().unwrap();
    swp.set_start_threshold(hwp.get_buffer_size().unwrap() - hwp.get_period_size().unwrap())
        .unwrap();
    pcm.sw_params(&swp).unwrap();
    io
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Parameter: ip:port");
        std::process::exit(1);
    }

    let ip_port = &args[1];
    let udpsocket = net::UdpSocket::bind("[::1]:0").unwrap();
    udpsocket.connect(ip_port).unwrap();

    let pcm = PCM::new("default", Direction::Playback, false).unwrap();
    let sndio = sound_init(&pcm, 48000, 1);

    let mut sndbuf: Vec<i16> = vec![];
    let mut buf_in = [0u8; 1 << 13];
    let mut buf: Vec<u8> = vec![];

    let mut allread = 0;
    loop {
        // reduce buffering-delay, empty TCP channel
        if let Ok(size) = udpsocket.recv(&mut buf_in) {
            allread += size;
            if size < 20 {
                break;
            }
        }
    }
    if allread % 2 == 1 {
        // odd
        buf.push(0);
    }

    let cmd = command_in::CmdIn::new();

    loop {
        if let Ok(size) = udpsocket.recv(&mut buf_in) {
            for &d in buf_in.iter().take(size) {
                buf.push(d);
            }
            let mut bufptr = 0;
            for i in (0..buf.len() - 1).step_by(2) {
                sndbuf.push((buf[i + 1] as i16) << 8);
                bufptr = i;
            }
            buf.drain(..bufptr);
            sndio.writei(&sndbuf[..]).unwrap();
            sndbuf.clear();
        }
        // command to server
        if let Some(data_in) = cmd.getstring() {
            // if new command ...
            let mut data = data_in;
            data.push('\n');
            data.push('\0');
            udpsocket.send(data.as_bytes()).unwrap();
        }
    }
}
