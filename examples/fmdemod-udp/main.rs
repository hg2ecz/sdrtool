use sdrtool::*;

fn main() {
    let samplerate = 2_400_000;
    let decimfactor = 10;
    let transition_bw = 1. / decimfactor as f32 / 2.;
    let window = Window::Hamming;

    let if_samplerate = samplerate / decimfactor;
    let audio_decimfactor = 5;
    let audiofreq = 10_000; // low pass audio filter
    let audio_transition_bw = audiofreq as f32 / if_samplerate as f32;

    let mut tcpcli = Sdrtcpcli::new("localhost:1234");
    let mut rfddc = Rfddc::new(samplerate, decimfactor, transition_bw, &window);
    let mut demod = Sdrdemod::new(
        if_samplerate,
        audio_decimfactor,
        audio_transition_bw,
        &window,
        audiofreq,
    );

    let mut udpsrv = Sndudpsrv::new(8888);
    eprintln!("F103.6<enter> # set RTL_SDR frequency to 103.6 MHz");
    eprintln!("f-0.3<enter>  # set mixer frequency to -0.3 MHz (+/- 1.08 MHz)");
    eprintln!("g-6<enter>    # set demodulator audio gain to -6 dB");
    let cmd = CmdIn::new(); // cmd from stdin

    loop {
        let rfdata = tcpcli.read_u8(); // rtl_sdr format
                                       // let rfdata = tcpcli.read_s16(); // miri_sdr format
        let rfif = rfddc.ddc(&rfdata); // mix & decimate
        let audio = demod.fmdemod(&rfif);
        let audio = demod.deemphasis_wfm(&audio, 50.0e-6);
        let audio = demod.decimate_audio(&audio);
        //write_stdout_i16(&audio);

        // write to sndclient & get cmd from sndclient --> Option<(char, f64)>
        if let Some((c, fval)) = udpsrv.write(&audio) {
            match c {
                'F' => set_rtlsdr(&mut tcpcli, c, fval),
                'f' => rfddc.mixer_setfreq(1_000_000. * fval, true),
                'g' => demod.set_gain(fval),
                _ => (),
            }
        }

        // set from serverconsole
        if let Some((c, fval)) = cmd.get_stdin() {
            match c {
                'F' => set_rtlsdr(&mut tcpcli, c, fval),
                'f' => rfddc.mixer_setfreq(1_000_000. * fval, true),
                'g' => demod.set_gain(fval),
                _ => (),
            }
        }
    }
}
