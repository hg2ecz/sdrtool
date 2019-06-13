#![allow(dead_code)]
use num_complex::Complex;

pub struct Sdrfilter {
    samplefirbuf: Vec<Complex<f32>>,      // for fir func
    sampledecimbuf: Vec<Complex<f32>>,    // for decimate func
    sampledecimmixbuf: Vec<Complex<f32>>, // for decimate_mix func
    coeff: Vec<Complex<f32>>,
    decimate: usize,
    decimate_pos: usize,
    oscillator: Complex<f64>,
    oscillator_phase: Complex<f64>,
}

impl Sdrfilter {
    pub fn new(coeff: &Vec<Complex<f32>>, decimate: usize) -> Sdrfilter {
        Sdrfilter {
             samplefirbuf: Vec::new(),
             sampledecimbuf: Vec::new(),
             sampledecimmixbuf: Vec::new(),
             coeff: coeff.clone(),
             decimate: decimate,
             decimate_pos: 0,
             oscillator: Complex::new(1., 0.),
             oscillator_phase: Complex::new(0., 0.),
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
        let fpsr: f64 = frequency / samplerate as f64;
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
        for i in (self.decimate_pos..self.sampledecimbuf.len()-self.coeff.len()).step_by(self.decimate) {
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
        for i in (self.decimate_pos..self.sampledecimmixbuf.len()-self.coeff.len()).step_by(self.decimate) {
            resvec.push ( self.sampledecimmixbuf[i..i+self.coeff.len()].iter().zip(&self.coeff).map(|(sa, co)| sa * co).sum() );
            pos=i;
        }
        self.sampledecimmixbuf.drain(..pos+self.decimate);
        return resvec;
    }
}
