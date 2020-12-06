pub fn set_rtlsdr(
    tcpcli: &mut super::Sdrtcpcli,
    rfddc: &mut super::Rfddc,
    demod: &mut super::Sdrdemod,
    cmd: Option<(char, f64)>,
) {
    if let Some((c, cf64)) = cmd {
        // if new command ...
        match c {
            'F' => {
                let mut cmdbytes: Vec<u8> = vec![0x01]; // RTL_TCP set Freq
                let f = (1_000_000. * cf64) as u32;
                cmdbytes.push((f >> 24) as u8);
                cmdbytes.push((f >> 16) as u8);
                cmdbytes.push((f >> 8) as u8);
                cmdbytes.push(f as u8);
                tcpcli.write_u8(&cmdbytes);
            }
            'f' => rfddc.mixer_setfreq(1_000_000. * cf64, true), // mix freq Hz
            'g' => demod.set_gain(cf64),                         // output gain ... decibel
            _ => {}
        }
    }
}
