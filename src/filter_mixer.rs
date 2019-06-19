#![allow(dead_code)]
use num_complex::Complex;
use std::f32::consts;

pub enum Window {
    Boxcar,
    Blackman,
    Hamming,
}

// transition_bw: 0.05
pub fn calc_coeff(cutoff_rate: f32, transition_bw: f32, window: Window) -> Vec<f32> {
    let taps_len=((4.0/transition_bw + 0.5) as usize) | 1; // odd
    let middle=taps_len/2;

    let mut sum=0.;
    let mut firtaps: Vec<f32> = vec![0.; taps_len];

    for i in 1..=middle {
        let rate=i as f32 / middle as f32 / 2. + 0.5;

        let wfuncval = match window {
            Window::Blackman =>
            // Explanation at Chapter 16 of dspguide.com, page 2
            // Blackman window has better stopband attentuation and passband ripple than Hamming, but it has slower rolloff.
                0.42-0.5*(2.*consts::PI*rate).cos() + 0.08*(4.*consts::PI*rate).cos(),
            Window::Hamming =>
                // Explanation at Chapter 16 of dspguide.com, page 2
                // Hamming window has worse stopband attentuation and passband ripple than Blackman, but it has faster rolloff.
                0.54-0.46*(2.*consts::PI*rate).cos(),
            _ => {
                println!("Bad type of filter! Exit.");
                std::process::exit(-1);
            },
        };

        firtaps[middle+i] = ((2.*consts::PI*cutoff_rate*i as f32).sin() / i as f32) * wfuncval;
        firtaps[middle-i] = firtaps[middle+i];
        sum += firtaps[middle-i] + firtaps[middle+i];
    }
    for i in 0..firtaps.len() {
        firtaps[i] /= sum;
    }
    return firtaps;
}

// ---------------------------------------------------------------

pub struct Sdrfilter {
    samplefirbuf: Vec<Complex<f32>>,      // for fir func
    sampledecimbuf: Vec<Complex<f32>>,    // for decimate func
    sampledecimmixbuf: Vec<Complex<f32>>, // for decimate_mix func
    coeff: Vec<f32>,
    decimate: usize,
    oscillator: Complex<f64>,
    oscillator_phase: Complex<f64>,
}

impl Sdrfilter {
    pub fn new(coeff: &Vec<f32>, decimate: usize) -> Sdrfilter {
        Sdrfilter {
             samplefirbuf: Vec::new(),
             sampledecimbuf: Vec::new(),
             sampledecimmixbuf: Vec::new(),
             coeff: coeff.clone(),
             decimate: decimate,
             oscillator: Complex::new(1., 0.),
             oscillator_phase: Complex::new(1., 0.),
        }
    }

    pub fn fir(&mut self, sample: &Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        self.samplefirbuf.extend(sample);
        let mut resvec = vec![];
        for i in 0..self.samplefirbuf.len()-self.coeff.len() {
            resvec.push ( self.samplefirbuf[i..i+self.coeff.len()].iter().zip(&self.coeff).map(|(sa, co)| sa * co).sum() );
        }
        self.samplefirbuf.drain(..self.samplefirbuf.len()-self.coeff.len());
        return resvec;
    }

    pub fn mixer_setfreq(&mut self, frequency: f64, samplerate: u32, phasereset: bool) {
        let fpsr: f64 = -2. * std::f64::consts::PI * frequency / samplerate as f64;
        self.oscillator_phase = Complex::new(fpsr.cos(), fpsr.sin());
        if phasereset {
            self.oscillator = Complex::new(1., 0.);
        }
    }

    pub fn mixer(&mut self, sample: &Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        let mut resvec = vec![];
        for i in 0..sample.len() {
            let tmp_osc32: Complex<f32> = Complex::new(self.oscillator.re as f32, self.oscillator.im as f32);
            resvec.push( sample[i] * tmp_osc32 );
            self.oscillator *= self.oscillator_phase;
        }
        return resvec;
    }

    pub fn decimate(&mut self, sample: &Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        let mut resvec = vec![];
        if self.decimate > self.coeff.len() { return resvec; }
        self.sampledecimbuf.extend(sample);
        let mut pos = 0;
        for i in (0..self.sampledecimbuf.len()-self.coeff.len()).step_by(self.decimate) {
            resvec.push ( self.sampledecimbuf[i..i+self.coeff.len()].iter().zip(&self.coeff).map(|(sa, co)| sa * co).sum() );
            pos=i;
        }
        self.sampledecimbuf.drain(..pos+self.decimate);
        return resvec;
    }

    pub fn decimate_mixer(&mut self, sample: &Vec<Complex<f32>>) -> Vec<Complex<f32>> {
        let mut resvec = vec![];
        if self.decimate > self.coeff.len() { return resvec; }
        for i in 0..sample.len() {
            let tmp_osc32: Complex<f32> = Complex::new(self.oscillator.re as f32, self.oscillator.im as f32);
            self.sampledecimmixbuf.push( sample[i] * tmp_osc32 );
            self.oscillator *= self.oscillator_phase;
        }
        let mut pos = 0;
        for i in (0..self.sampledecimmixbuf.len()-self.coeff.len()).step_by(self.decimate) {
            resvec.push ( self.sampledecimmixbuf[i..i+self.coeff.len()].iter().zip(&self.coeff).map(|(sa, co)| sa * co).sum() );
            pos=i;
        }
        self.sampledecimmixbuf.drain(..pos+self.decimate);
        return resvec;
    }
}
