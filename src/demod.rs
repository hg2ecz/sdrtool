#![allow(dead_code)]
use num_complex::Complex;

pub struct Sdrdemod {
    fm_z1: Complex<f32>,
    fm_z2: Complex<f32>,

    sampledecimbuf: Vec<f32>, // for decimate_mix func
    coeff: Vec<f32>,
    decimate: usize,
    decimate_pos: usize,

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
            deemp_last_output: 0.,
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