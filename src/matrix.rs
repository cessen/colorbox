//! Functions and types for building and working with color transform matrices.
//!
//! For the sake of precision during construction, all the functions in
//! this module work with `f64` matrices.  However, after construction
//! you generally want to convert to `f32` for actual use.  The
//! `to_3x3_f32()` and `to_4x4_f32()` functions are provided for that
//! purpose.
//!
//! When generating e.g. lookup tables, it may be useful to directly
//! transform colors by matrices generated with this module.
//! `transform_color()` is provided for that use case.

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

/// Inverse of `rgb_to_xyz_matrix()`.
pub fn xyz_to_rgb_matrix(chroma: Chromaticities) -> Matrix {
    inverse(rgb_to_xyz_matrix(chroma)).unwrap()
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
    multiply(rgb_to_xyz_matrix(src), xyz_to_rgb_matrix(dst))
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
    let from_abc = inverse(to_abc).unwrap();

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
    compose(&[to_abc, w_scale, from_abc])
}

/// Creates a matrix that simply scales each component of a color by the
/// given scale values.
pub fn scale_matrix(scale: [f64; 3]) -> Matrix {
    [
        [scale[0], 0.0, 0.0],
        [0.0, scale[1], 0.0],
        [0.0, 0.0, scale[2]],
    ]
}

/// Calculates the inverse of a matrix.
///
/// Returns `None` is the matrix is not invertible.
pub fn inverse(m: Matrix) -> Option<Matrix> {
    // Misc computations that are used more than once below.
    // Just for optimization, no meaning behind these.
    let a = (m[1][1] * m[2][2]) - (m[2][1] * m[1][2]);
    let b = m[1][0] * m[2][2];
    let c = m[1][2] * m[2][0];
    let d = m[1][0] * m[2][1];

    // Determinant.
    let det = m[0][0] * a - m[0][1] * (b - c) + m[0][2] * (d - m[1][1] * m[2][0]);

    if det == 0.0 {
        // No inverse.
        return None;
    }

    let invdet = 1.0 / det;

    Some([
        [
            a * invdet,
            (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * invdet,
            (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * invdet,
        ],
        [
            (c - b) * invdet,
            (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * invdet,
            (m[1][0] * m[0][2] - m[0][0] * m[1][2]) * invdet,
        ],
        [
            (d - m[2][0] * m[1][1]) * invdet,
            (m[2][0] * m[0][1] - m[0][0] * m[2][1]) * invdet,
            (m[0][0] * m[1][1] - m[1][0] * m[0][1]) * invdet,
        ],
    ])
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

/// Composes matrices together as a sequence of transforms.
///
/// Panics if `matrices` is empty.
#[inline(always)]
pub fn compose(matrices: &[Matrix]) -> Matrix {
    assert!(!matrices.is_empty());

    let mut temp = matrices[0];
    for mat in &matrices[1..] {
        temp = multiply(temp, *mat);
    }
    temp
}

/// Transforms a color by a matrix.
#[inline]
pub fn transform_color(color: [f64; 3], m: Matrix) -> [f64; 3] {
    let mut c = [0.0f64; 3];

    c[0] = (color[0] * m[0][0]) + (color[1] * m[0][1]) + (color[2] * m[0][2]);
    c[1] = (color[0] * m[1][0]) + (color[1] * m[1][1]) + (color[2] * m[1][2]);
    c[2] = (color[0] * m[2][0]) + (color[1] * m[2][1]) + (color[2] * m[2][2]);

    c
}

/// Converts to a 3x3 f32 matrix with a flattened layout.
pub fn to_3x3_f32(m: Matrix) -> [f32; 9] {
    [
        m[0][0] as f32,
        m[0][1] as f32,
        m[0][2] as f32,
        m[1][0] as f32,
        m[1][1] as f32,
        m[1][2] as f32,
        m[2][0] as f32,
        m[2][1] as f32,
        m[2][2] as f32,
    ]
}

/// Converts to a 4x4 f32 matrix with a flattened layout.
pub fn to_4x4_f32(m: Matrix) -> [f32; 16] {
    [
        m[0][0] as f32,
        m[0][1] as f32,
        m[0][2] as f32,
        0.0,
        m[1][0] as f32,
        m[1][1] as f32,
        m[1][2] as f32,
        0.0,
        m[2][0] as f32,
        m[2][1] as f32,
        m[2][2] as f32,
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ]
}

//-------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{matrix_max_diff, vec_max_diff};

    #[test]
    fn rgb_to_xyz_test() {
        let mat = rgb_to_xyz_matrix(crate::chroma::REC709);
        assert!(
            matrix_max_diff(
                mat,
                [
                    [0.4123907992, 0.3575843393, 0.1804807884],
                    [0.2126390058, 0.7151686787, 0.0721923153],
                    [0.0193308187, 0.1191947797, 0.9505321522],
                ]
            ) < 0.000_000_001
        );

        let mat = rgb_to_xyz_matrix(crate::chroma::ADOBE_WIDE_GAMUT_RGB);
        assert!(
            matrix_max_diff(
                mat,
                [
                    [0.7165007167, 0.1010205743, 0.1467743852],
                    [0.2587282430, 0.7246823149, 0.0165894420],
                    [0.0, 0.0512118189, 0.7738927835],
                ]
            ) < 0.000_000_001
        );

        let mat = rgb_to_xyz_matrix(crate::chroma::ACES_AP0);
        assert!(
            matrix_max_diff(
                mat,
                // Matrix from official ACES repo:
                // https://github.com/ampas/aces-dev/blob/master/transforms/ctl/README-MATRIX.md
                [
                    [0.9525523959, 0.0000000000, 0.0000936786],
                    [0.3439664498, 0.7281660966, -0.0721325464],
                    [0.0000000000, 0.0000000000, 1.0088251844],
                ]
            ) < 0.000_000_001
        );

        let mat = rgb_to_xyz_matrix(crate::chroma::ACES_AP1);
        assert!(
            matrix_max_diff(
                mat,
                // Matrix from official ACES repo:
                // https://github.com/ampas/aces-dev/blob/master/transforms/ctl/README-MATRIX.md
                [
                    [0.6624541811, 0.1340042065, 0.1561876870],
                    [0.2722287168, 0.6740817658, 0.0536895174],
                    [-0.0055746495, 0.0040607335, 1.0103391003],
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

        let mat = rgb_to_rgb_matrix(crate::chroma::ACES_AP1, crate::chroma::ACES_AP0);
        assert!(
            matrix_max_diff(
                mat,
                // Matrix from official ACES repo:
                // https://github.com/ampas/aces-dev/blob/master/transforms/ctl/README-MATRIX.md
                [
                    [0.6954522414, 0.1406786965, 0.1638690622],
                    [0.0447945634, 0.8596711185, 0.0955343182],
                    [-0.0055258826, 0.0040252103, 1.0015006723],
                ]
            ) < 0.000_000_001
        );
    }

    #[test]
    fn chromatic_adaptation_test_01() {
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
    fn chromatic_adaptation_test_02() {
        let mat = xyz_chromatic_adaptation_matrix(
            crate::chroma::ACES_AP0.w,
            crate::chroma::REC709.w,
            AdaptationMethod::Bradford,
        );
        assert!(
            matrix_max_diff(
                mat,
                // Matrix verified against official ACES repo:
                // https://github.com/ampas/aces-dev/blob/master/transforms/ctl/README-MATRIX.md
                [
                    [0.9872240087, -0.0061132286, 0.0159532883],
                    [-0.0075983718, 1.0018614847, 0.0053300358],
                    [0.0030725771, -0.0050959615, 1.0816806031],
                ]
            ) < 0.000_000_001
        );

        let mat = xyz_chromatic_adaptation_matrix(
            crate::chroma::PROPHOTO.w,
            crate::chroma::REC709.w,
            AdaptationMethod::XYZScale,
        );
        assert!(
            matrix_max_diff(
                mat,
                [
                    [0.9857463844, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                    [0.0, 0.0, 1.3202463042],
                ]
            ) < 0.000_000_001
        );

        let mat = xyz_chromatic_adaptation_matrix(
            crate::chroma::PROPHOTO.w,
            crate::chroma::REC709.w,
            AdaptationMethod::Hunt,
        );
        assert!(
            matrix_max_diff(
                mat,
                [
                    [0.9844773043, -0.0546989286, 0.0677939921],
                    [-0.0060082931, 1.0047945956, 0.0012105812],
                    [0.0, 0.0, 1.3202463042],
                ]
            ) < 0.000_000_001
        );

        let mat = xyz_chromatic_adaptation_matrix(
            crate::chroma::PROPHOTO.w,
            crate::chroma::REC709.w,
            AdaptationMethod::Bradford,
        );
        assert!(
            matrix_max_diff(
                mat,
                [
                    [0.9555118600, -0.0230733576, 0.0633120466],
                    [-0.0283250104, 1.0099425961, 0.0210553666],
                    [0.0123293595, -0.0205364519, 1.3307307257],
                ]
            ) < 0.000_000_001
        );
    }

    #[test]
    fn matrix_inverse_test() {
        let mat = rgb_to_xyz_matrix(crate::chroma::ACES_AP0);
        let inv = inverse(mat).unwrap();

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

    #[test]
    fn compose_test() {
        let m1 = [[2.0, 3.0, 4.0], [5.0, 6.0, 7.0], [8.0, 9.0, 10.0]];
        let m2 = [[64.0, 16.0, 8.0], [4.0, 2.0, 1.0], [0.5, 0.25, 0.125]];
        let m3 = [[5.0, 62.4, 7.7], [4.0, 23.0, 2.1], [12.66, 8.3, 42.0]];

        let out1 = multiply(multiply(m1, m2), m3);
        let out2 = compose(&[m1, m2, m3]);

        for (n1, n2) in out1.iter().flatten().zip(out2.iter().flatten()) {
            assert_eq!(n1, n2);
        }
    }
}
