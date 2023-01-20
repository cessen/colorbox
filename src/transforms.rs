//! Useful transforms to use as building blocks.

/// Open Color IO compatible fixed function transforms.
///
/// The transforms in this module reproduce some of the fixed-function
/// transforms built in to Open Color IO.  They are implemented to
/// exactly reproduce the behavior of the OCIO functions.
pub mod ocio {
    /// RGB -> HSV conversion.
    ///
    /// H is in the range `[0.0, 1.0)`, S is in the range `[0.0, 2.0)`
    /// with 1.0 as full saturation and > 1.0 as over-saturated (i.e.
    /// outside the input RGB colorspace), and V is unbounded.
    pub fn rgb_to_hsv(rgb: [f64; 3]) -> [f64; 3] {
        let [red, grn, blu] = rgb;

        let rgb_min = red.min(grn.min(blu));
        let rgb_max = red.max(grn.max(blu));
        let delta = rgb_max - rgb_min;

        let mut val = rgb_max;
        let mut sat = 0.0;
        let mut hue = 0.0;

        if delta != 0.0 {
            // Sat
            if rgb_max != 0.0 {
                sat = delta / rgb_max;
            }

            // Hue
            if red == rgb_max {
                hue = (grn - blu) / delta;
            } else if grn == rgb_max {
                hue = 2.0 + (blu - red) / delta;
            } else {
                hue = 4.0 + (red - grn) / delta;
            }

            if hue < 0.0 {
                hue += 6.0;
            }

            hue *= 1.0 / 6.0;
        }

        // Handle extended range inputs.
        if rgb_min < 0.0 {
            val += rgb_min;
        }

        if -rgb_min > rgb_max {
            sat = (rgb_max - rgb_min) / -rgb_min;
        }

        [hue, sat, val]
    }

    /// HSV -> RGB conversion.
    ///
    /// Input H is treated as wrapping in the range [0.0, 1.0), and S is
    /// clipped to the range [0.0, 2.0) before processing.
    pub fn hsv_to_rgb(hsv: [f64; 3]) -> [f64; 3] {
        const MAX_SAT: f64 = 1.999;

        let hue = (hsv[0] - hsv[0].floor()) * 6.0;
        let sat = hsv[1].clamp(0.0, MAX_SAT);
        let val = hsv[2];

        let red = ((hue - 3.0).abs() - 1.0).clamp(0.0, 1.0);
        let grn = (2.0 - (hue - 2.0).abs()).clamp(0.0, 1.0);
        let blu = (2.0 - (hue - 4.0).abs()).clamp(0.0, 1.0);

        let mut rgb_max = val;
        let mut rgb_min = val * (1.0 - sat);

        // Handle extended range inputs.
        if sat > 1.0 {
            rgb_min = val * (1.0 - sat) / (2.0 - sat);
            rgb_max = val - rgb_min;
        }
        if val < 0.0 {
            rgb_min = val / (2.0 - sat);
            rgb_max = val - rgb_min;
        }

        let delta = rgb_max - rgb_min;

        [
            red * delta + rgb_min,
            grn * delta + rgb_min,
            blu * delta + rgb_min,
        ]
    }
}

pub mod oklab {
    use crate::matrix::{transform_color, Matrix};

    /// CIE XYZ -> OkLab.
    ///
    /// Note that OkLab assumes a D65 whitepoint, so input colors with a
    /// different whitepoint should be adapted to that before being
    /// passed.
    #[inline]
    pub fn xyz_d65_to_oklab(xyz: [f64; 3]) -> [f64; 3] {
        const M1: Matrix = [
            [0.8189330101, 0.3618667424, -0.1288597137],
            [0.0329845436, 0.9293118715, 0.0361456387],
            [0.0482003018, 0.2643662691, 0.6338517070],
        ];
        const M2: Matrix = [
            [0.2104542553, 0.7936177850, -0.0040720468],
            [1.9779984951, -2.4285922050, 0.4505937099],
            [0.0259040371, 0.7827717662, -0.8086757660],
        ];

        let lms_linear = transform_color(xyz, M1);

        // `abs` and `signum` keep it from choking on negative values.
        let lms_nonlinear = [
            lms_linear[0].abs().powf(1.0 / 3.0) * lms_linear[0].signum(),
            lms_linear[1].abs().powf(1.0 / 3.0) * lms_linear[1].signum(),
            lms_linear[2].abs().powf(1.0 / 3.0) * lms_linear[2].signum(),
        ];

        transform_color(lms_nonlinear, M2)
    }

    /// OkLab -> XYZ.
    ///
    /// Note that OkLab assumes a D65 whitepoint, so output colors have
    /// that whitepoint and should be adapted if desired.
    #[inline]
    pub fn oklab_to_xyz_d65(oklab: [f64; 3]) -> [f64; 3] {
        const M1_INV: Matrix = [
            [1.2270138511035211, -0.5577999806518222, 0.2812561489664678],
            [
                -0.040580178423280586,
                1.11225686961683,
                -0.07167667866560119,
            ],
            [-0.0763812845057069, -0.4214819784180127, 1.5861632204407947],
        ];
        const M2_INV: Matrix = [
            [0.9999999984505197, 0.3963377921737678, 0.21580375806075883],
            [
                1.0000000088817607,
                -0.10556134232365633,
                -0.0638541747717059,
            ],
            [
                1.0000000546724108,
                -0.08948418209496575,
                -1.2914855378640917,
            ],
        ];

        let lms_nonlinear = transform_color(oklab, M2_INV);

        // `abs` and `signum` keep it from choking on negative values in `xyz_d65_to_oklab()`.
        let lms_linear = [
            lms_nonlinear[0].abs().powf(3.0) * lms_nonlinear[0].signum(),
            lms_nonlinear[1].abs().powf(3.0) * lms_nonlinear[1].signum(),
            lms_nonlinear[2].abs().powf(3.0) * lms_nonlinear[2].signum(),
        ];

        transform_color(lms_linear, M1_INV)
    }

    #[inline(always)]
    fn oklab_to_oklch(lab: [f64; 3]) -> [f64; 3] {
        let c = ((lab[1] * lab[1]) + (lab[2] * lab[2])).sqrt();
        let h = lab[2].atan2(lab[1]);

        [lab[0], c, h]
    }

    #[inline(always)]
    fn oklch_to_oklab(lch: [f64; 3]) -> [f64; 3] {
        let a = lch[1] * lch[2].cos();
        let b = lch[1] * lch[2].sin();

        [lch[0], a, b]
    }

    /// Uses OkLab space to clip out-of-gamut rgb colors in a relatively
    /// pleasing way.
    ///
    /// The to and from matrices convert between the rgb color space and
    /// CIE XYZ space.  Since OkLab assumes a D65 whitepoint, these
    /// matrices should include whitepoint adaptation to/from D65 if
    /// needed.
    ///
    /// - `rgb`: the input color to be clipped.  Assumed to be linear.
    /// - `to_xyz_d65_mat`: transform matrix from RGB -> XYZ.  Should
    ///   include adaptation to a D65 whitepoint if needed.
    /// - `from_xyz_d65_mat`: transform matrix from XYZ -> RGB.  Should
    ///   include adaptation from a D65 whitepoint if needed.
    /// - `method`: the gamut clipping method to use.
    ///
    /// Returns the clipped RGB color.
    pub fn rgb_gamut_clip(
        rgb: [f64; 3],
        channel_max: Option<f64>,
        to_xyz_d65_mat: Matrix,
        from_xyz_d65_mat: Matrix,
    ) -> [f64; 3] {
        let from_rgb = |rgb| oklab_to_oklch(xyz_d65_to_oklab(transform_color(rgb, to_xyz_d65_mat)));
        let to_rgb = |lch| transform_color(oklab_to_xyz_d65(oklch_to_oklab(lch)), from_xyz_d65_mat);
        let is_in_gamut = |rgb: [f64; 3]| {
            if let Some(m) = channel_max {
                rgb[0] >= 0.0
                    && rgb[1] >= 0.0
                    && rgb[2] >= 0.0
                    && rgb[0] <= m
                    && rgb[1] <= m
                    && rgb[2] < m
            } else {
                rgb[0] >= 0.0 && rgb[1] >= 0.0 && rgb[2] >= 0.0
            }
        };

        // Early out: if we're already in gamut, just return the original rgb value.
        if is_in_gamut(rgb) {
            return rgb;
        }

        let mut lch_from = from_rgb(rgb);

        // Projection target is equal-luminance gray, but with
        // luminance clipped to [0.0, max].
        let mut lch_target = [
            if let Some(m) = channel_max {
                lch_from[0].min(from_rgb([m, m, m])[0])
            } else {
                lch_from[0]
            }
            .max(0.0),
            0.0,
            lch_from[2],
        ];

        // Clip negative luminance to zero.
        if lch_from[0] <= 0.0 {
            return [0.0, 0.0, 0.0];
        }

        // Use binary search to iteratively find the clip point.
        for _ in 0..32 {
            let lch_mid = [
                (lch_from[0] + lch_target[0]) * 0.5,
                (lch_from[1] + lch_target[1]) * 0.5,
                (lch_from[2] + lch_target[2]) * 0.5,
            ];

            if is_in_gamut(to_rgb(lch_mid)) {
                lch_target = lch_mid;
            } else {
                lch_from = lch_mid;
            }

            // Termination criteria.
            if (lch_from[1] - lch_target[1]).abs() < 0.001
                && ((lch_from[0] / lch_target[0]) - 1.0).abs() < 0.001
            {
                break;
            }
        }

        to_rgb(lch_target)
    }

    //---------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn xyz_d65_to_oklab_01() {
            const TEST_VECS: &[([f64; 3], [f64; 3])] = &[
                ([0.95, 1.0, 1.089], [1.0, 0.0, 0.0]),
                ([1.0, 0.0, 0.0], [0.45, 1.236, -0.019]),
                ([0.0, 1.0, 0.0], [0.922, -0.671, 0.263]),
                ([0.0, 0.0, 1.0], [0.153, -1.415, -0.449]),
            ];
            for (v1, v2) in TEST_VECS.iter().copied() {
                let r1 = xyz_d65_to_oklab(v1);
                for i in 0..3 {
                    assert!((r1[i] - v2[i]).abs() < 0.002);
                }
            }
        }

        #[test]
        fn oklab_to_xyz_d65_01() {
            const TEST_VECS: &[([f64; 3], [f64; 3])] = &[
                ([0.95, 1.0, 1.089], [1.0, 0.0, 0.0]),
                ([1.0, 0.0, 0.0], [0.45, 1.236, -0.019]),
                ([0.0, 1.0, 0.0], [0.922, -0.671, 0.263]),
                ([0.0, 0.0, 1.0], [0.153, -1.415, -0.449]),
            ];
            for (v1, v2) in TEST_VECS.iter().copied() {
                let r2 = oklab_to_xyz_d65(v2);
                for i in 0..3 {
                    assert!((v1[i] - r2[i]).abs() < 0.002);
                }
            }
        }

        #[test]
        fn oklab_to_oklch_01() {
            const TEST_VECS: &[[f64; 3]] = &[
                [2.0, 1.0, 0.5],
                [2.0, 0.65, 1.3],
                [2.0, 0.7, 0.2],
                [-2.0, 0.7, 0.2],
                [2.0, -0.7, 0.2],
                [2.0, 0.7, -0.2],
            ];
            for v in TEST_VECS.iter().copied() {
                let v2 = oklch_to_oklab(oklab_to_oklch(v));
                for i in 0..3 {
                    assert!((v[i] - v2[i]).abs() < 0.00001);
                }
            }
        }
    }
}
