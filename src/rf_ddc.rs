#![allow(dead_code)]
use num_complex::Complex;

// ---------------------------------------------------------------

pub struct Rfddc {
    samplerate: u32,
    samplefirbuf: Vec<Complex<f32>>,      // for fir func
    sampledecimbuf: Vec<Complex<f32>>,    // for decimate func
    sampledecimmixbuf: Vec<Complex<f32>>, // for decimate_mix func
    coeff: Vec<f32>,
    decimfactor: usize,
    is_mix: bool,
    oscillator: Complex<f64>,
    oscillator_phase: Complex<f64>,
}

impl Rfddc {
        pub fn new(samplerate: u32, decimfactor: u32, transition_bw: f32, window: &super::fir_util::Window) -> Rfddc {

        let cutoff_rate = 1./decimfactor as f32;
        let coeff = super::fir_util::calc_coeff(cutoff_rate, transition_bw, window);

        Rfddc {
             samplerate,
             samplefirbuf: Vec::new(),
             sampledecimbuf: Vec::new(),
             sampledecimmixbuf: Vec::new(),
             coeff,
             decimfactor: decimfactor as usize,
             is_mix: false,
             oscillator: Complex::new(1., 0.),
             oscillator_phase: Complex::new(1., 0.),
        }
    }

    pub fn fir(&mut self, sample: &[Complex<f32>]) -> Vec<Complex<f32>> {
        self.samplefirbuf.extend(sample);
        let mut resvec = vec![];
        for i in 0..self.samplefirbuf.len()-self.coeff.len() {
            resvec.push ( self.coeff.iter().zip(&self.sampledecimbuf[i..]).map(|(co, sa)| sa * co).sum() );
        }
        self.samplefirbuf.drain(..self.samplefirbuf.len()-self.coeff.len());
        resvec
    }

    pub fn mixer_setfreq(&mut self, frequency: f64, phasereset: bool) {
        self.is_mix = frequency > 0.;
        let fpsr: f64 = -2. * std::f64::consts::PI * frequency / f64::from(self.samplerate);
        self.oscillator_phase = Complex::new(fpsr.cos(), fpsr.sin());
        if phasereset {
            self.oscillator = Complex::new(1., 0.);
        }
    }

    pub fn mixer(&mut self, sample: &[Complex<f32>]) -> Vec<Complex<f32>> {
        let mut resvec = vec![];
        for s in sample {
            let tmp_osc32: Complex<f32> = Complex::new(self.oscillator.re as f32, self.oscillator.im as f32);
            resvec.push( s * tmp_osc32 );
            self.oscillator *= self.oscillator_phase;
        }
        resvec
    }

    pub fn decimate(&mut self, sample: &[Complex<f32>]) -> Vec<Complex<f32>> {
        let mut resvec = vec![];
        if self.decimfactor > self.coeff.len() { return resvec; }
        self.sampledecimbuf.extend(sample);
        let mut pos = 0;
        for i in (0..self.sampledecimbuf.len()-self.coeff.len()).step_by(self.decimfactor) {
            resvec.push ( self.coeff.iter().zip(&self.sampledecimbuf[i..]).map(|(co, sa)| sa * co).sum() );
            pos=i;
        }
        self.sampledecimbuf.drain(..pos+self.decimfactor);
        resvec
    }

    // mixer & decimate
    pub fn ddc(&mut self, sample: &[Complex<f32>]) -> Vec<Complex<f32>> {
        let mut resvec = vec![];
        if self.decimfactor > self.coeff.len() { return resvec; }
        if self.is_mix {
            for s in sample {
                let tmp_osc32: Complex<f32> = Complex::new(self.oscillator.re as f32, self.oscillator.im as f32);
                resvec.push( s * tmp_osc32 );
                self.oscillator *= self.oscillator_phase;
            }
        } else {
            self.sampledecimmixbuf.extend(sample);
        }
        let mut pos = 0;
        for i in (0..self.sampledecimmixbuf.len()-self.coeff.len()).step_by(self.decimfactor) {
            resvec.push ( self.coeff.iter().zip(&self.sampledecimmixbuf[i..]).map(|(co, sa)| co * sa).sum() );
            pos=i;
        }
        self.sampledecimmixbuf.drain(..pos+self.decimfactor);

        resvec
    }
}
