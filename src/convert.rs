pub fn f32_to_i16(sound: &[f32]) -> Vec<i16> {
    sound.iter().map(|&x| x as i16).collect()
}
