pub fn set_rtlsdr(tcpcli: &mut super::Sdrtcpcli, cmd: char, fval: f64) {
    if cmd == 'F' {
        let mut cmdbytes = [0x01, 0, 0, 0, 0]; // RTL_TCP set Freq
        let f = (1_000_000. * fval) as u32;
        cmdbytes[1] = (f >> 24) as u8;
        cmdbytes[2] = (f >> 16) as u8;
        cmdbytes[3] = (f >> 8) as u8;
        cmdbytes[4] = f as u8;
        tcpcli.write_u8(&cmdbytes);
    }
}
