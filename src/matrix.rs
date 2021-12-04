//! Functions and types for building and working with color transform matrices.

use crate::chroma::Chromaticities;

/// A 3x3 color transform matrix.
pub type Matrix = [[f64; 3]; 3];

/// Computes a matrix to transform colors from the specified RGB color
/// space to CIE 1931 XYZ color space.
///
/// The resulting matrix is a straightforward transform matrix into XYZ
/// color space, and does not attempt any chromatic adaptation.  This
/// means, for example, that an RGB value of `1,1,1` will map to a color
/// with chromaticity equal to the specified white point.
///
/// - `chroma` is the chromaticities of the RGB color space.
pub fn rgb_to_xyz_matrix(chroma: Chromaticities) -> Matrix {
    // X and Z values of RGB value (1, 1, 1), or "white".
    let x = chroma.w.0 / chroma.w.1;
    let z = (1.0 - chroma.w.0 - chroma.w.1) / chroma.w.1;

    // Scale factors for matrix rows.
    let d = chroma.r.0 * (chroma.b.1 - chroma.g.1)
        + chroma.b.0 * (chroma.g.1 - chroma.r.1)
        + chroma.g.0 * (chroma.r.1 - chroma.b.1);

    let sr = (x * (chroma.b.1 - chroma.g.1)
        - chroma.g.0 * ((chroma.b.1 - 1.0) + chroma.b.1 * (x + z))
        + chroma.b.0 * ((chroma.g.1 - 1.0) + chroma.g.1 * (x + z)))
        / d;

    let sg = (x * (chroma.r.1 - chroma.b.1)
        + chroma.r.0 * ((chroma.b.1 - 1.0) + chroma.b.1 * (x + z))
        - chroma.b.0 * ((chroma.r.1 - 1.0) + chroma.r.1 * (x + z)))
        / d;

    let sb = (x * (chroma.g.1 - chroma.r.1)
        - chroma.r.0 * ((chroma.g.1 - 1.0) + chroma.g.1 * (x + z))
        + chroma.g.0 * ((chroma.r.1 - 1.0) + chroma.r.1 * (x + z)))
        / d;

    // Assemble the matrix.
    let mut mat = [[0.0; 3]; 3];

    mat[0][0] = sr * chroma.r.0;
    mat[0][1] = sg * chroma.g.0;
    mat[0][2] = sb * chroma.b.0;

    mat[1][0] = sr * chroma.r.1;
    mat[1][1] = sg * chroma.g.1;
    mat[1][2] = sb * chroma.b.1;

    mat[2][0] = sr * (1.0 - chroma.r.0 - chroma.r.1);
    mat[2][1] = sg * (1.0 - chroma.g.0 - chroma.g.1);
    mat[2][2] = sb * (1.0 - chroma.b.0 - chroma.b.1);

    mat
}

/// Computes a matrix to transform colors from one RGB color space to
/// another.
///
/// Like `rgb_to_xyz_matrix()`, the resulting matrix is just a
/// straightforward color space transform, and doesn't attempt any
/// chromatic adaptation.  So if the white points differ, `1,1,1` in the
/// `src` space will not map to `1,1,1` in the `dst` space.
#[inline]
pub fn rgb_to_rgb_matrix(src: Chromaticities, dst: Chromaticities) -> Matrix {
    let a = rgb_to_xyz_matrix(src);
    let b = invert(rgb_to_xyz_matrix(dst)).unwrap();

    multiply(a, b)
}

/// Chromatic adaptation methods.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AdaptationMethod {
    /// Scale the coordinates directly in CIE 1931 XYZ color space.
    /// This is generally considered to be a poor method, but may be
    /// useful in some situations.
    XYZScale,

    /// Use the Hunt-Pointer-Estevez LMS transform and Von Kries scaling.
    Hunt,

    /// Use the Bradford RGB transform and Von Kries scaling.
    Bradford,
}

/// Computes a matrix to chromatically adapt CIE 1931 XYZ colors
/// from one white point to another.
///
/// The matrices computed by this function can only be validly applied to
/// colors in CIE 1931 XYZ color space.
///
/// The computed matrix essentially "moves" colors from one white point
/// to another.  For example, a `src_w` of D65 and a `dst_w` of E would
/// create a matrix that if applied to a point at D65 would result in a
/// point at E.
///
/// - `src_w`: the source white point's chromaticity coordinates (in CIE
///            1931 xy).
/// - `dst_w`: the destination white point's chromaticity coordinates (in
///            CIE 1931 xy).
/// - `method`: the adaptation method to use.
pub fn xyz_chromatic_adaptation_matrix(
    src_w: (f64, f64),
    dst_w: (f64, f64),
    method: AdaptationMethod,
) -> Matrix {
    // Identity.
    const IDENTITY: Matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    // The Hunt-Pointer-Estevez transformation matrix.
    const TO_LMS_HUNT: Matrix = [
        [0.38971, 0.68898, -0.07868],
        [-0.22981, 1.18340, 0.04641],
        [0.0, 0.0, 1.0],
    ];

    // The Bradford transformation matrix.
    const TO_RGB_BRADFORD: Matrix = [
        [0.8951, 0.2664, -0.1614],
        [-0.7502, 1.7135, 0.0367],
        [0.0389, -0.0685, 1.0296],
    ];

    // Decide what space to do the Von Kries scaling it.
    // We're calling the resulting space "ABC" here, since
    // whether it's e.g. LMS, RGB, or whatever depends on
    // the space chosen.
    let to_abc = match method {
        AdaptationMethod::XYZScale => IDENTITY,
        AdaptationMethod::Hunt => TO_LMS_HUNT,
        AdaptationMethod::Bradford => TO_RGB_BRADFORD,
    };
    let from_abc = invert(to_abc).unwrap();

    // Compute the white points' XYZ values.
    let src_w_xyz = [src_w.0 / src_w.1, 1.0, (1.0 - src_w.0 - src_w.1) / src_w.1];
    let dst_w_xyz = [dst_w.0 / dst_w.1, 1.0, (1.0 - dst_w.0 - dst_w.1) / dst_w.1];

    // Compute the white points' ABC values.
    let src_w_abc = transform_color(src_w_xyz, to_abc);
    let dst_w_abc = transform_color(dst_w_xyz, to_abc);

    // Compute the Von Kries matrix to scale the ABC values appropriately.
    let w_scale = [
        [dst_w_abc[0] / src_w_abc[0], 0.0, 0.0],
        [0.0, dst_w_abc[1] / src_w_abc[1], 0.0],
        [0.0, 0.0, dst_w_abc[2] / src_w_abc[2]],
    ];

    // Combine the matrices.
    multiply(multiply(to_abc, w_scale), from_abc)
}

/// Calculates the inverse of a matrix.
///
/// Returns `None` is the matrix is not invertible.
pub fn invert(m: Matrix) -> Option<Matrix> {
    let mut s = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let mut t = m;

    // Forward elimination
    for i in 0..2 {
        let mut pivot = i;
        let mut pivotsize = t[i][i];

        if pivotsize < 0.0 {
            pivotsize = -pivotsize;
        }

        for j in (i + 1)..3 {
            let mut tmp = t[j][i];

            if tmp < 0.0 {
                tmp = -tmp;
            }

            if tmp > pivotsize {
                pivot = j;
                pivotsize = tmp;
            }
        }

        if pivotsize == 0.0 {
            return None;
        }

        if pivot != i {
            for j in 0..3 {
                let mut tmp = t[i][j];
                t[i][j] = t[pivot][j];
                t[pivot][j] = tmp;

                tmp = s[i][j];
                s[i][j] = s[pivot][j];
                s[pivot][j] = tmp;
            }
        }

        for j in (i + 1)..3 {
            let f = t[j][i] / t[i][i];

            for k in 0..3 {
                t[j][k] -= f * t[i][k];
                s[j][k] -= f * s[i][k];
            }
        }
    }

    // Backward substitution
    for i in (0..3).rev() {
        let f = t[i][i];

        if t[i][i] == 0.0 {
            return None;
        }

        for j in 0..3 {
            t[i][j] /= f;
            s[i][j] /= f;
        }

        for j in 0..i {
            let f = t[j][i];

            for k in 0..3 {
                t[j][k] -= f * t[i][k];
                s[j][k] -= f * s[i][k];
            }
        }
    }

    Some(s)
}

/// Multiplies two matrices together.
///
/// The result is a matrix that is equivalent to first
/// transforming by `a` and then by `b`.
#[inline]
pub fn multiply(a: Matrix, b: Matrix) -> Matrix {
    let mut c = [[0.0f64; 3]; 3];

    c[0][0] = (b[0][0] * a[0][0]) + (b[0][1] * a[1][0]) + (b[0][2] * a[2][0]);
    c[0][1] = (b[0][0] * a[0][1]) + (b[0][1] * a[1][1]) + (b[0][2] * a[2][1]);
    c[0][2] = (b[0][0] * a[0][2]) + (b[0][1] * a[1][2]) + (b[0][2] * a[2][2]);

    c[1][0] = (b[1][0] * a[0][0]) + (b[1][1] * a[1][0]) + (b[1][2] * a[2][0]);
    c[1][1] = (b[1][0] * a[0][1]) + (b[1][1] * a[1][1]) + (b[1][2] * a[2][1]);
    c[1][2] = (b[1][0] * a[0][2]) + (b[1][1] * a[1][2]) + (b[1][2] * a[2][2]);

    c[2][0] = (b[2][0] * a[0][0]) + (b[2][1] * a[1][0]) + (b[2][2] * a[2][0]);
    c[2][1] = (b[2][0] * a[0][1]) + (b[2][1] * a[1][1]) + (b[2][2] * a[2][1]);
    c[2][2] = (b[2][0] * a[0][2]) + (b[2][1] * a[1][2]) + (b[2][2] * a[2][2]);

    c
}

/// Transforms a color by a matrix.
#[inline]
fn transform_color(color: [f64; 3], m: Matrix) -> [f64; 3] {
    let mut c = [0.0f64; 3];

    c[0] = (color[0] * m[0][0]) + (color[1] * m[0][1]) + (color[2] * m[0][2]);
    c[1] = (color[0] * m[1][0]) + (color[1] * m[1][1]) + (color[2] * m[1][2]);
    c[2] = (color[0] * m[2][0]) + (color[1] * m[2][1]) + (color[2] * m[2][2]);

    c
}

//-------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_max_diff(a: [f64; 3], b: [f64; 3]) -> f64 {
        let mut max_diff: f64 = 0.0;
        for (aa, bb) in a.iter().zip(b.iter()) {
            let diff = (aa - bb).abs();
            max_diff = max_diff.max(diff);
        }
        max_diff
    }

    fn matrix_max_diff(a: Matrix, b: Matrix) -> f64 {
        let mut max_diff: f64 = 0.0;
        for (aa, bb) in a.iter().flatten().zip(b.iter().flatten()) {
            let diff = (aa - bb).abs();
            max_diff = max_diff.max(diff);
        }
        max_diff
    }

    #[test]
    fn rgb_to_xyz_test() {
        let mat = rgb_to_xyz_matrix(crate::chroma::ACES_AP0);

        assert!(
            matrix_max_diff(
                mat,
                [
                    [0.9525523959, 0.0000000000, 0.0000936786],
                    [0.3439664498, 0.7281660966, -0.0721325464],
                    [0.0000000000, 0.0000000000, 1.0088251844],
                ]
            ) < 0.000_000_001
        );
    }

    #[test]
    fn rgb_to_rgb_test() {
        let mat = rgb_to_rgb_matrix(crate::chroma::REC709, crate::chroma::ACES_AP0);

        assert!(
            matrix_max_diff(
                mat,
                [
                    [0.4329305201, 0.3753843595, 0.1893780579],
                    [0.0894131371, 0.8165330211, 0.1030219928],
                    [0.0191617131, 0.1181520660, 0.9422169143],
                ]
            ) < 0.000_000_001
        );
    }

    #[test]
    fn chromatic_adaptation_test() {
        let to_xyz = rgb_to_xyz_matrix(crate::chroma::REC709);
        let adapt_xyz = xyz_chromatic_adaptation_matrix(
            crate::chroma::REC709.w,
            (1.0 / 3.0, 1.0 / 3.0),
            AdaptationMethod::XYZScale,
        );
        let adapt_hunt = xyz_chromatic_adaptation_matrix(
            crate::chroma::REC709.w,
            (1.0 / 3.0, 1.0 / 3.0),
            AdaptationMethod::Hunt,
        );
        let adapt_bradford = xyz_chromatic_adaptation_matrix(
            crate::chroma::REC709.w,
            (1.0 / 3.0, 1.0 / 3.0),
            AdaptationMethod::Bradford,
        );

        let white_1 = transform_color([1.0, 1.0, 1.0], multiply(to_xyz, adapt_xyz));
        let white_2 = transform_color([1.0, 1.0, 1.0], multiply(to_xyz, adapt_hunt));
        let white_3 = transform_color([1.0, 1.0, 1.0], multiply(to_xyz, adapt_bradford));

        assert!(vec_max_diff(white_1, [1.0, 1.0, 1.0]) < 0.000_000_001);
        assert!(vec_max_diff(white_2, [1.0, 1.0, 1.0]) < 0.000_000_001);
        assert!(vec_max_diff(white_3, [1.0, 1.0, 1.0]) < 0.000_000_001);
    }

    #[test]
    fn matrix_invert_test() {
        let mat = rgb_to_xyz_matrix(crate::chroma::ACES_AP0);
        let inv = invert(mat).unwrap();

        assert!(
            matrix_max_diff(
                multiply(mat, inv),
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0],]
            ) < 0.000_000_001
        );
    }

    #[test]
    fn matrix_multiply() {
        let rec709_mat = rgb_to_xyz_matrix(crate::chroma::REC709);
        let aces_mat = rgb_to_xyz_matrix(crate::chroma::ACES_AP0);
        let combined_mat = multiply(rec709_mat, aces_mat);

        let vec1 = [0.0, 0.0, 0.0];
        let vec2 = [1.0, 0.0, 0.0];
        let vec3 = [0.0, 1.0, 0.0];
        let vec4 = [0.0, 0.0, 1.0];
        let vec5 = [1.0, 1.0, 1.0];

        assert!(
            vec_max_diff(
                transform_color(transform_color(vec1, rec709_mat), aces_mat),
                transform_color(vec1, combined_mat),
            ) < 0.000_000_000_000_001
        );
        assert!(
            vec_max_diff(
                transform_color(transform_color(vec2, rec709_mat), aces_mat),
                transform_color(vec2, combined_mat),
            ) < 0.000_000_000_000_001
        );
        assert!(
            vec_max_diff(
                transform_color(transform_color(vec3, rec709_mat), aces_mat),
                transform_color(vec3, combined_mat),
            ) < 0.000_000_000_000_001
        );
        assert!(
            vec_max_diff(
                transform_color(transform_color(vec4, rec709_mat), aces_mat),
                transform_color(vec4, combined_mat),
            ) < 0.000_000_000_000_001
        );
        assert!(
            vec_max_diff(
                transform_color(transform_color(vec5, rec709_mat), aces_mat),
                transform_color(vec5, combined_mat),
            ) < 0.000_000_000_000_001
        );
    }
}
