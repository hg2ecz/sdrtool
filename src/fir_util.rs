use std::f32::consts;

pub enum Window {
    Boxcar,
    Blackman,
    Hamming,
}

// transition_bw: 0.05
pub fn calc_coeff(cutoff_rate: f32, transition_bw: f32, window: &Window) -> Vec<f32> {
    let taps_len = ((4.0 / transition_bw + 0.5) as usize) | 1; // odd
    let middle = taps_len / 2;

    let mut sum = 0.;
    let mut firtaps: Vec<f32> = vec![0.; taps_len];

    for i in 1..=middle {
        let rate = i as f32 / middle as f32 / 2. + 0.5;

        let wfuncval = match window {
            Window::Blackman =>
            // Explanation at Chapter 16 of dspguide.com, page 2
            // Blackman window has better stopband attentuation and passband ripple than Hamming, but it has slower rolloff.
            {
                0.42 - 0.5 * (2. * consts::PI * rate).cos() + 0.08 * (4. * consts::PI * rate).cos()
            }
            Window::Hamming =>
            // Explanation at Chapter 16 of dspguide.com, page 2
            // Hamming window has worse stopband attentuation and passband ripple than Blackman, but it has faster rolloff.
            {
                0.54 - 0.46 * (2. * consts::PI * rate).cos()
            }
            _ => {
                println!("Bad type of filter! Exit.");
                std::process::exit(-1);
            }
        };

        firtaps[middle + i] =
            ((2. * consts::PI * cutoff_rate * i as f32).sin() / i as f32) * wfuncval;
        firtaps[middle - i] = firtaps[middle + i];
        sum += firtaps[middle - i] + firtaps[middle + i];
    }
    for ftap in &mut firtaps {
        *ftap /= sum;
    }
    eprintln!("Fir LEN: {}", firtaps.len());
    firtaps
}
