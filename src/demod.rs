#![allow(dead_code)]
use num_complex::Complex;

/** Demodulator (FM, ...), deemphasis, audio decimator

A minimal FM demodulator example (RF data from rtl_tcp -s 2.4M -f 103.3M):

```
use sdrtool::*;

fn main() {
    let rfcoeff = calc_coeff(240_000./2_400_000., 0.05, Window::Hamming);
    let audiocoeff = calc_coeff(15_000./240_000., 0.05, Window::Hamming);

    let mut tcpcli = Sdrtcpcli::new("localhost:1234");
    let mut rfddc = Rfddc::new(&rfcoeff, 10);       // coeffvec & decimate_factor
    let mut demod = Sdrdemod::new(&audiocoeff, 5);     // coeffvec & decimate_factor

    loop {
        let rfdata = tcpcli.read_u8();
        let rfif = rfddc.ddc(&rfdata);
        let audio = demod.fmdemod(&rfif);
        let audio = demod.deemphasis_wfm(&audio, 50.0e-6, 240_000);
        let audio = demod.decimate_audio(&audio);
        write_stdout_i16(&audio);
    }
}
```
**/
pub struct Sdrdemod {
    fm_z1: Complex<f32>,
    fm_z2: Complex<f32>,

    sampledecimbuf: Vec<f32>, // for decimate_mix func
    coeff: Vec<f32>,
    decimate: usize,
    decimate_pos: usize,

    demod_gain: f32,
    deemp_last_output: f32,
}

impl Sdrdemod {
    pub fn new(coeff: &Vec<f32>, decimate: usize) -> Sdrdemod {
        Sdrdemod {
            fm_z1: Complex::new(0., 0.),
            fm_z2: Complex::new(0., 0.),
            sampledecimbuf: vec![],
            coeff: coeff.clone(),
            decimate: decimate,
            decimate_pos: 0,
            demod_gain: 0.5,
            deemp_last_output: 0.,
        }
    }

    pub fn set_gain(&mut self, gain_decibel: f64) {
        self.demod_gain = 10.0f32.powf(gain_decibel as f32/20.) / 2.;
    }

    pub fn fmdemod(&mut self, sample: &Vec<Complex<f32>>) -> Vec<f32> {
        let mut output: Vec<f32> = vec![];
        for &signal in sample {
            output.push( (   self.fm_z1.re * (signal.im - self.fm_z2.im)
                           - self.fm_z1.im * (signal.re - self.fm_z2.re)
                         ) * self.demod_gain );
            self.fm_z2 = self.fm_z1;
            self.fm_z1 = signal;
        }
        return output;
    }

    pub fn decimate_audio(&mut self, sample: &Vec<f32>) -> Vec<f32> {
        let mut resvec = vec![];
        if self.decimate > self.coeff.len() { return resvec; }
        self.sampledecimbuf.extend(sample);
        let mut pos = 0;
        for i in (self.decimate_pos..self.sampledecimbuf.len()-self.coeff.len()).step_by(self.decimate) {
            resvec.push ( self.sampledecimbuf[i..i+self.coeff.len()].iter().zip(&self.coeff).map(|(sa, co)| sa * co).sum() );
            pos=i;
        }
        self.sampledecimbuf.drain(..pos+self.decimate);
        return resvec;
    }

    pub fn deemphasis_wfm(&mut self, input: &Vec<f32>, tau: f32, sample_rate: u32) -> Vec<f32> {
        // typical time constant (tau) values:
        // WFM transmission in USA: 75 us -> tau = 75e-6
        // WFM transmission in EU:  50 us -> tau = 50e-6
        // More info at: http://www.cliftonlaboratories.com/fm_receivers_and_de-emphasis.htm
        // Simulate in octave: tau=75e-6; dt=1/48000; alpha = dt/(tau+dt); freqz([alpha],[1 -(1-alpha)])
        let dt = 1.0/sample_rate as f32;
        let alpha = dt/(tau+dt);
        let mut output = vec![];
        output.push ( alpha*input[0]+(1.-alpha)*self.deemp_last_output );
        for i in 1..input.len() { //@deemphasis_wfm_ff
            output.push( alpha*input[i]+(1.-alpha)*output[i-1] ); //this is the simplest IIR LPF
        }
        self.deemp_last_output = *output.last().unwrap();
        return output;
    }
}
