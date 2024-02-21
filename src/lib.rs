//! A low-level toolbox for working with color.

pub mod chroma;
pub mod formats;
pub mod lut;
pub mod matrix;
pub mod tables;
pub mod transfer_functions;
pub mod transforms;

//-------------------------------------------------------------
// Misc functions for use in tests throughout the crate.

#[cfg(test)]
fn vec_max_diff(a: [f64; 3], b: [f64; 3]) -> f64 {
    let mut max_diff: f64 = 0.0;
    for (aa, bb) in a.iter().zip(b.iter()) {
        let diff = (aa - bb).abs();
        max_diff = max_diff.max(diff);
    }
    max_diff
}

#[cfg(test)]
fn matrix_max_diff(a: crate::matrix::Matrix, b: crate::matrix::Matrix) -> f64 {
    let mut max_diff: f64 = 0.0;
    for (aa, bb) in a.iter().flatten().zip(b.iter().flatten()) {
        let diff = (aa - bb).abs();
        max_diff = max_diff.max(diff);
    }
    max_diff
}
