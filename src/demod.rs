#![allow(dead_code)]
use num_complex::Complex;

pub struct Sdrdemod {
    fm_z1: Complex<f32>,
    fm_z2: Complex<f32>,

    sampledecimbuf: Vec<f32>, // for decimate_mix func
    coeff: Vec<f32>,
    decimate: usize,
    decimate_pos: usize,
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
        }
    }

    pub fn fmdemod(&mut self, sample: &Vec<Complex<f32>>) -> Vec<f32> {
        let mut output: Vec<f32> = vec![];
        for &signal in sample {
            output.push( (   self.fm_z1.re * (signal.im - self.fm_z2.im)
                           - self.fm_z1.im * (signal.re - self.fm_z2.re)
                         ) * 1./3. );
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
}
