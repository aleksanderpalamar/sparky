use std::f32::consts::PI;

pub const PEAK: f32 = 1.6;
const REACH: f32 = 2.5;

pub fn scale(distance: f32, pitch: f32) -> f32 {
    let ratio = distance.abs() / (pitch * REACH);
    match ratio < 1.0 {
        true => 1.0 + (PEAK - 1.0) * ((ratio * PI).cos() + 1.0) / 2.0,
        false => 1.0,
    }
}

pub fn targets(pointer: Option<f32>, centers: &[f32], pitch: f32) -> Vec<f32> {
    centers
        .iter()
        .map(|center| pointer.map_or(1.0, |x| scale(x - center, pitch)))
        .collect()
}
