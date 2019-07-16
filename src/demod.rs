#![allow(dead_code)]
use num_complex::Complex;

/** Demodulator (FM, ...), deemphasis, audio decimator

A minimal FM demodulator example (RF data from rtl_tcp -s 2.4M -f 103.3M):

```
use sdrtool::*;

fn main() {
    let samplerate: u32 = 2_400_000;
    let decimfactor: u32 = 10;
    let transition_bw = 0.05;
    let window = Window::Hamming;

    let if_samplerate = samplerate/decimfactor;
    let audio_decimfactor = 5;
    let audiofreq = 15_000;    // low pass filter

    let mut tcpcli = Sdrtcpcli::new("localhost:1234");
    let mut rfddc = Rfddc::new(samplerate, decimfactor, transition_bw, &window);
    let mut demod = Sdrdemod::new(if_samplerate, audio_decimfactor, transition_bw, &window, audiofreq);

    loop {
        let rfdata = tcpcli.read_u8();
        let rfif = rfddc.ddc(&rfdata);
        let audio = demod.fmdemod(&rfif);
        let audio = demod.deemphasis_wfm(&audio, 50.0e-6);
        let audio = demod.decimate_audio(&audio);
        write_stdout_i16(&audio);
    }
}
```
**/
pub struct Sdrdemod {
    fm_z1: Complex<f32>,
    fm_z2: Complex<f32>,

    samplerate: u32,
    sampledecimbuf: Vec<f32>, // for decimate_mix func
    coeff: Vec<f32>,
    decimfactor: usize,
    decimate_pos: usize,

    demod_gain: f32,
    deemp_last_output: f32,

    audiofreq: u32, // fir lowpass filter
}

impl Sdrdemod {
    pub fn new(samplerate: u32, decimfactor: usize, transition_bw: f32, window: &super::Window, audiofreq: u32) -> Sdrdemod {

        let cutoff_rate = 1./decimfactor as f32;
        let coeff = super::calc_coeff(cutoff_rate, transition_bw, window);

        Sdrdemod {
            fm_z1: Complex::new(0., 0.),
            fm_z2: Complex::new(0., 0.),
            samplerate,
            sampledecimbuf: vec![],
            coeff,
            decimfactor,
            decimate_pos: 0,
            demod_gain: 0.5,
            deemp_last_output: 0.,
	    audiofreq,
        }
    }

    pub fn set_gain(&mut self, gain_decibel: f64) {
        self.demod_gain = 10.0f32.powf(gain_decibel as f32/20.) / 2.;
    }

    pub fn fmdemod(&mut self, sample: &[Complex<f32>]) -> Vec<f32> {
        let mut output: Vec<f32> = vec![];
        for &signal in sample {
            output.push( (   self.fm_z1.re * (signal.im - self.fm_z2.im)
                           - self.fm_z1.im * (signal.re - self.fm_z2.re)
                         ) * self.demod_gain );
            self.fm_z2 = self.fm_z1;
            self.fm_z1 = signal;
        }
        output
    }

    pub fn decimate_audio(&mut self, sample: &[f32]) -> Vec<f32> {
        let mut resvec = vec![];
        if self.decimfactor > self.coeff.len() { return resvec; }
        self.sampledecimbuf.extend(sample);
        let mut pos = 0;
        for i in (self.decimate_pos..self.sampledecimbuf.len()-self.coeff.len()).step_by(self.decimfactor) {
            resvec.push ( self.sampledecimbuf[i..i+self.coeff.len()].iter().zip(&self.coeff).map(|(sa, co)| sa * co).sum() );
            pos=i;
        }
        self.sampledecimbuf.drain(..pos+self.decimfactor);
        resvec
    }

    pub fn deemphasis_wfm(&mut self, input: &[f32], tau: f32) -> Vec<f32> {
        // typical time constant (tau) values:
        // WFM transmission in USA: 75 us -> tau = 75e-6
        // WFM transmission in EU:  50 us -> tau = 50e-6
        // More info at: http://www.cliftonlaboratories.com/fm_receivers_and_de-emphasis.htm
        // Simulate in octave: tau=75e-6; dt=1/48000; alpha = dt/(tau+dt); freqz([alpha],[1 -(1-alpha)])
        let dt = 1.0/self.samplerate as f32;
        let alpha = dt/(tau+dt);
        let mut output = vec![];
        output.push ( alpha*input[0]+(1.-alpha)*self.deemp_last_output );
        for i in 1..input.len() { //@deemphasis_wfm_ff
            output.push( alpha*input[i]+(1.-alpha)*output[i-1] ); //this is the simplest IIR LPF
        }
        self.deemp_last_output = *output.last().unwrap();
        output
    }
}
