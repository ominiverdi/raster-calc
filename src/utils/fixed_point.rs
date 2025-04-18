// src/utils/fixed_point.rs
pub fn to_fixed_point(data: &[f32], scale_factor: i32, nodata_value: i16) -> Vec<i16> {
    data.iter()
        .map(|&value| {
            if value == -999.0 {
                nodata_value
            } else {
                // Clamp to avoid overflow and scale
                let clamped = value.max(-0.9999).min(0.9999);
                (clamped * scale_factor as f32).round() as i16
            }
        })
        .collect()
}