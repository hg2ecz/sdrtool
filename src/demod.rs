#![allow(dead_code)]
use num_complex::Complex;

pub struct Sdrdemod {
    fm_z1: Complex<f32>,
    fm_z2: Complex<f32>,
}

impl Sdrdemod {
    pub fn new() -> Sdrdemod {
        Sdrdemod {
            fm_z1: Complex::new(0., 0.),
            fm_z2: Complex::new(0., 0.),
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
}
