//! Various known transfer functions.

/// sRGB gamma.
pub mod srgb {
    /// Linear -> sRGB
    #[inline]
    pub fn from_linear(n: f32) -> f32 {
        if n < 0.003_130_8 {
            n * 12.92
        } else {
            (1.055 * n.powf(1.0 / 2.4)) - 0.055
        }
    }

    /// sRGB -> Linear
    #[inline]
    pub fn to_linear(n: f32) -> f32 {
        if n < 0.04045 {
            n / 12.92
        } else {
            ((n + 0.055) / 1.055).powf(2.4)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            assert_eq!(from_linear(0.0), 0.0);
            assert!((from_linear(1.0) - 1.0).abs() < 0.000_001);
        }

        #[test]
        fn to_linear_test() {
            assert_eq!(to_linear(0.0), 0.0);
            assert!((to_linear(1.0) - 1.0).abs() < 0.000_001);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = i as f32 / 1023.0;
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_001);
            }
        }
    }
}

/// Rec.709 and Rec.2020 gamma.
pub mod rec709 {
    // We use high-precision versions of the constants here
    // so that it works for Rec.2020 as well.
    const A: f32 = 1.09929682680944;
    const B: f32 = 0.01805396851080;
    const C: f32 = A - 1.0;

    /// Linear -> sRGB
    #[inline]
    pub fn from_linear(n: f32) -> f32 {
        if n < B {
            n * 4.5
        } else {
            (A * n.powf(0.45)) - C
        }
    }

    /// sRGB -> Linear
    #[inline]
    pub fn to_linear(n: f32) -> f32 {
        if n < (B * 4.5) {
            n / 4.5
        } else {
            ((n + C) / A).powf(1.0 / 0.45)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            assert_eq!(from_linear(0.0), 0.0);
            assert!((from_linear(1.0) - 1.0).abs() < 0.000_001);
        }

        #[test]
        fn to_linear_test() {
            assert_eq!(to_linear(0.0), 0.0);
            assert!((to_linear(1.0) - 1.0).abs() < 0.000_001);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = i as f32 / 1023.0;
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_001);
            }
        }
    }
}

/// Perceptual Quantizer from Rec.2100.
///
/// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
/// mapping.  It is a transfer function between linear
/// [0.0, `LUMINANCE_MAX`] (in cd/m^2) and non-linear [0.0, 1.0].
pub mod pq {
    /// The maximum allowed luminance of linear values, in cd/m^2.
    pub const LUMINANCE_MAX: f32 = 10000.0;

    const M1: f32 = 2610.0 / 16384.0;
    const M2: f32 = 2523.0 / 4096.0 * 128.0;
    const C1: f32 = 3424.0 / 4096.0;
    const C2: f32 = 2413.0 / 4096.0 * 32.0;
    const C3: f32 = 2392.0 / 4096.0 * 32.0;

    /// Linear -> PQ.
    ///
    /// Input is in the range [0, `LUMINANCE_MAX`], representing display
    /// luminance in cd/m^2.
    /// Output is in the range [0.0, 1.0].
    #[inline(always)]
    pub fn from_linear(n: f32) -> f32 {
        let n = n * (1.0 / LUMINANCE_MAX);

        let n_m1 = n.powf(M1);
        ((C1 + (C2 * n_m1)) / (1.0 + (C3 * n_m1))).powf(M2)
    }

    /// PQ -> Linear.
    ///
    /// Input is in the range [0.0, 1.0].
    /// Output is in the range [0, `LUMINANCE_MAX`], representing display
    /// luminance in cd/m^2.
    #[inline(always)]
    pub fn to_linear(n: f32) -> f32 {
        let n_1_m2 = n.powf(1.0 / M2);
        let linear = ((n_1_m2 - C1).max(0.0) / (C2 - (C3 * n_1_m2))).powf(1.0 / M1);

        linear * LUMINANCE_MAX
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            assert!((from_linear(0.0) - 0.0).abs() < 0.000_001);
            assert!((from_linear(LUMINANCE_MAX) - 1.0).abs() < 0.000_001);
        }

        #[test]
        fn to_linear_test() {
            assert!((to_linear(0.0) - 0.0).abs() < 0.000_001);
            assert!((to_linear(1.0) - LUMINANCE_MAX).abs() < 0.000_001);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = i as f32 / 1023.0;
                dbg!(n);
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_1);
            }
        }
    }
}

/// Hybrid Log-Gamma from Rec.2100.
pub mod hlg {
    const A: f32 = 0.17883277;
    const B: f32 = 1.0 - (4.0 * A);

    /// Linear -> HLG.
    ///
    /// Input and output are both [0.0, 1.0].
    #[inline]
    pub fn from_linear(n: f32) -> f32 {
        let c = 0.5 - (A * (4.0 * A).ln()); // Should be a `const`, but can't because of `ln()`.

        if n <= (1.0 / 12.0) {
            (3.0 * n).sqrt()
        } else {
            A * (12.0 * n - B).ln() + c
        }
    }

    /// HLG -> Linear.
    ///
    /// Input and output are both [0.0, 1.0].
    #[inline]
    pub fn to_linear(n: f32) -> f32 {
        let c = 0.5 - (A * (4.0 * A).ln()); // Should be a `const`, but can't because of `ln()`.

        if n <= 0.5 {
            (n * n) / 3.0
        } else {
            (((n - c) / A).exp() + B) / 12.0
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            assert!((from_linear(0.0) - 0.0).abs() < 0.000_001);
            assert!((from_linear(1.0) - 1.0).abs() < 0.000_001);
        }

        #[test]
        fn to_linear_test() {
            assert!((to_linear(0.0) - 0.0).abs() < 0.000_001);
            assert!((to_linear(1.0) - 1.0).abs() < 0.000_001);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = i as f32 / 1023.0;
                dbg!(n);
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_001);
            }
        }
    }
}

/// Sony's S-Log (original).
///
/// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
/// mapping.  It is a transfer function between "scene linear" and
/// normalized "code values".  For example, scene-linear 0.0
/// maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
/// 1.0 maps to a much greater than 1.0 scene linear value.
pub mod sony_slog1 {
    /// The normalized code value of scene-linear 0.0.
    pub const CV_BLACK: f32 = 0.088251315;

    /// The normalized code value of camera sensor saturation.
    pub const CV_SATURATION: f32 = SLOG_WHITE;

    /// The scene-linear value of normalized code value 0.0.
    pub const LINEAR_MIN: f32 = -0.014279289;

    /// The scene-linear value of normalized code value 1.0.
    pub const LINEAR_MAX: f32 = 9.737593;

    /// Misc internal constants used on the S-Log formulas.
    const A: f32 = 0.432699;
    const B: f32 = 0.037584;
    const C: f32 = 0.616596;
    const SLOG_BLACK: f32 = 64.0 / 1023.0;
    const SLOG_WHITE: f32 = 940.0 / 1023.0;

    /// From scene linear to (normalized) code values.
    ///
    /// For example, to get 10-bit code values do
    /// `from_linear(scene_linear_in) * 1023.0`
    #[inline]
    pub fn from_linear(x: f32) -> f32 {
        let x = x / 0.9;

        let y = (A * (x + B).log10() + C) + 0.03;

        // Map 0.0 and 1.0 to "code value" black and white levels,
        // respectively.
        (y * (SLOG_WHITE - SLOG_BLACK)) + SLOG_BLACK
    }

    /// From (normalized) code values to scene linear.
    ///
    /// For example, if using 10-bit code values do
    /// `to_linear(10_bit_cv_in / 1023.0)`
    #[inline]
    pub fn to_linear(x: f32) -> f32 {
        // Map "code value" black and white levels to 0.0 and 1.0,
        // respectively.
        let x = (x - SLOG_BLACK) / (SLOG_WHITE - SLOG_BLACK);

        let y = 10.0f32.powf((x - C - 0.03) / A) - B;

        y * 0.9
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            // Invariants from page 6 of "S-Log White Paper 1.12.3" from
            // Sony, October 23rd 2009.
            assert!((from_linear(0.0) - (90.0 / 1023.0)).abs() < 0.001);
            assert!((from_linear(0.02) - (167.0 / 1023.0)).abs() < 0.001);
            assert!((from_linear(0.18) - (394.0 / 1023.0)).abs() < 0.001);
            assert!((from_linear(0.9) - (636.0 / 1023.0)).abs() < 0.001);
            assert!((from_linear(7.192) - (974.0 / 1023.0)).abs() < 0.001);
        }

        #[test]
        fn to_linear_test() {
            // Invariants from page 6 of "S-Log White Paper 1.12.3" from
            // Sony, October 23rd 2009.
            assert!((to_linear(90.0 / 1023.0) - 0.0).abs() < 0.001);
            assert!((to_linear(167.0 / 1023.0) - 0.02).abs() < 0.001);
            assert!((to_linear(394.0 / 1023.0) - 0.18).abs() < 0.001);
            assert!((to_linear(636.0 / 1023.0) - 0.9).abs() < 0.003);
            assert!((to_linear(974.0 / 1023.0) - 7.192).abs() < 0.03);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = (i as f32 / 1023.0) * (LINEAR_MAX - LINEAR_MIN) + LINEAR_MIN;
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_01);
            }
        }
    }
}

/// Sony's S-Log2.
///
/// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
/// mapping.  It is a transfer function between "scene linear" and
/// normalized "code values".  For example, scene-linear 0.0
/// maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
/// 1.0 maps to a much greater than 1.0 scene linear value.
pub mod sony_slog2 {
    /// Misc internal constants used on the S-Log2 formulas.
    const SLOG2_BLACK: f32 = 64.0 / 1023.0;
    const SLOG2_WHITE: f32 = 940.0 / 1023.0;

    /// The normalized code value of scene-linear 0.0.
    pub const CV_BLACK: f32 = 0.088251315;

    /// The normalized code value of camera sensor saturation.
    pub const CV_SATURATION: f32 = SLOG2_WHITE;

    /// The scene-linear value of normalized code value 0.0.
    pub const LINEAR_MIN: f32 = -0.026210632;

    /// The scene-linear value of normalized code value 1.0.
    pub const LINEAR_MAX: f32 = 13.758276;

    /// From scene linear to (normalized) code values.
    ///
    /// For example, to get 10-bit code values do
    /// `from_linear(scene_linear_in) * 1023.0`
    #[inline]
    pub fn from_linear(x: f32) -> f32 {
        let x = x / 0.9;

        // Mapping curve.
        let y = if x < 0.0 {
            x * 3.538_812_785_388_13 + 0.030_001_222_851_889_303
        } else {
            (0.432699 * (155.0 * x / 219.0 + 0.037584).log10() + 0.616596) + 0.03
        };

        // Map 0.0 and 1.0 to "code value" black and white levels,
        // respectively.
        (y * (SLOG2_WHITE - SLOG2_BLACK)) + SLOG2_BLACK
    }

    /// From (normalized) code values to scene linear.
    ///
    /// For example, if using 10-bit code values do
    /// `to_linear(10_bit_cv_in / 1023.0)`
    #[inline]
    pub fn to_linear(x: f32) -> f32 {
        // Map "code value" black and white levels to 0.0 and 1.0,
        // respectively.
        let x = (x - SLOG2_BLACK) / (SLOG2_WHITE - SLOG2_BLACK);

        // Mapping curve.
        let y = if x < 0.030_001_222_851_889_303 {
            (x - 0.030_001_222_851_889_303) / 3.538_812_785_388_13
        } else {
            219.0 * (10.0f32.powf((x - 0.03 - 0.616596) / 0.432699) - 0.037584) / 155.0
        };

        y * 0.9
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            // Invariants from page 6 of "S-Log2 Technical Paper v1.0" from
            // Sony, June 6th 2012.
            assert!((from_linear(0.0) - (90.0 / 1023.0)).abs() < 0.001);
            assert!((from_linear(0.18) - (347.0 / 1023.0)).abs() < 0.001);
            assert!((from_linear(0.9) - (582.0 / 1023.0)).abs() < 0.001);
        }

        #[test]
        fn to_linear_test() {
            // Invariants from page 6 of "S-Log2 Technical Paper v1.0" from
            // Sony, June 6th 2012.
            assert!((to_linear(90.0 / 1023.0) - 0.0).abs() < 0.001);
            assert!((to_linear(347.0 / 1023.0) - 0.18).abs() < 0.001);
            assert!((to_linear(582.0 / 1023.0) - 0.9).abs() < 0.001);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = (i as f32 / 1023.0) * (LINEAR_MAX - LINEAR_MIN) + LINEAR_MIN;
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_01);
            }
        }
    }
}

/// Sony's S-Log3.
///
/// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
/// mapping.  It is a transfer function between "scene linear" and
/// normalized "code values".  For example, scene-linear 0.0
/// maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
/// 1.0 maps to a much greater than 1.0 scene linear value.
pub mod sony_slog3 {
    /// The normalized code value of scene-linear 0.0.
    pub const CV_BLACK: f32 = 0.092864126;

    /// The scene-linear value of normalized code value 0.0.
    pub const LINEAR_MIN: f32 = -0.014023696;

    /// The scene-linear value of normalized code value 1.0.
    pub const LINEAR_MAX: f32 = 38.420933;

    /// From scene linear to (normalized) code values.
    ///
    /// For example, to get 10-bit code values do
    /// `from_linear(scene_linear_in) * 1023.0`
    pub fn from_linear(x: f32) -> f32 {
        if x < 0.01125000 {
            (x * (171.2102946929 - 95.0) / 0.01125000 + 95.0) / 1023.0
        } else {
            (420.0 + ((x + 0.01) / (0.18 + 0.01)).log10() * 261.5) / 1023.0
        }
    }

    /// From (normalized) code values to scene linear.
    ///
    /// For example, if using 10-bit code values do
    /// `to_linear(10_bit_cv_in / 1023.0)`
    pub fn to_linear(x: f32) -> f32 {
        if x < (171.2102946929 / 1023.0) {
            (x * 1023.0 - 95.0) * 0.01125000 / (171.2102946929 - 95.0)
        } else {
            (10.0f32.powf((x * 1023.0 - 420.0) / 261.5)) * (0.18 + 0.01) - 0.01
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            // Invariants from page 6 of "Technical Summary for
            // S-Gamut3.Cine/S-Log3 and S-Gamut3/S-Log3", from Sony.
            assert!((from_linear(0.0) - (95.0 / 1023.0)).abs() < 0.001);
            assert!((from_linear(0.18) - (420.0 / 1023.0)).abs() < 0.001);
            assert!((from_linear(0.9) - (598.0 / 1023.0)).abs() < 0.001);
        }

        #[test]
        fn to_linear_test() {
            // Invariants from page 6 of "Technical Summary for
            // S-Gamut3.Cine/S-Log3 and S-Gamut3/S-Log3", from Sony.
            assert!((to_linear(95.0 / 1023.0) - 0.0).abs() < 0.001);
            assert!((to_linear(420.0 / 1023.0) - 0.18).abs() < 0.001);
            assert!((to_linear(598.0 / 1023.0) - 0.9).abs() < 0.001);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = (i as f32 / 1023.0) * (LINEAR_MAX - LINEAR_MIN) + LINEAR_MIN;
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_05);
            }
        }
    }
}

/// Canon Log (original).
///
/// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
/// mapping.  It is a transfer function between "scene linear" and a
/// nonlinear [0.0, 1.0] range.  For example, scene-linear 0.0 maps to
/// `NONLINEAR_BLACK` (which is > 0.0), and a nonlinear value of 1.0 maps
/// to a much greater than 1.0 scene-linear value.
pub mod canon_log1 {
    /// The nonlinear value of scene-linear 0.0.
    pub const NONLINEAR_BLACK: f32 = 0.12512247;

    /// The scene-linear value of nonlinear value 0.0.
    pub const LINEAR_MIN: f32 = -0.087466866;

    /// The scene-linear value of nonlinear value 1.0.
    pub const LINEAR_MAX: f32 = 8.295911;

    const A: f32 = 0.45310179;
    const B: f32 = 10.1596;
    const C: f32 = 0.12512248;

    /// Linear -> Canon Log 2
    pub fn from_linear(x: f32) -> f32 {
        if x < 0.0 {
            -A * (1.0 - (B * x)).log10() + C
        } else {
            A * (1.0 + (B * x)).log10() + C
        }
    }

    /// Canon Log 2 -> Linear
    pub fn to_linear(x: f32) -> f32 {
        if x < C {
            -(10.0f32.powf((C - x) / A) - 1.0) / B
        } else {
            (10.0f32.powf((x - C) / A) - 1.0) / B
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            // Invariants from page 9 of "Canon Log Gamma Curves -
            // Description of the Canon Log, Canon Log 2 and Canon Log 3
            // Gamma Curves", from Canon, November 1st 2018.
            assert!((from_linear(0.0) - 0.125).abs() < 0.001);
            assert!((from_linear(0.2) - 0.343).abs() < 0.001);
            assert!((from_linear(1.0) - 0.6).abs() < 0.001);
            assert!((from_linear(8.0) - 0.993).abs() < 0.001);
        }

        #[test]
        fn to_linear_test() {
            // Invariants from page 9 of "Canon Log Gamma Curves -
            // Description of the Canon Log, Canon Log 2 and Canon Log 3
            // Gamma Curves", from Canon, November 1st 2018.
            assert!((to_linear(0.125) - 0.0).abs() < 0.001);
            assert!((to_linear(0.343) - 0.2).abs() < 0.001);
            assert!((to_linear(0.6) - 1.0).abs() < 0.002);
            assert!((to_linear(0.993) - 8.0).abs() < 0.003);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = (i as f32 / 1023.0) * (LINEAR_MAX - LINEAR_MIN) + LINEAR_MIN;
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_01);
            }
        }
    }
}

/// Canon Log 2.
///
/// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
/// mapping.  It is a transfer function between "scene linear" and a
/// nonlinear [0.0, 1.0] range.  For example, scene-linear 0.0 maps to
/// `NONLINEAR_BLACK` (which is > 0.0), and a nonlinear value of 1.0 maps
/// to a much greater than 1.0 scene-linear value.
pub mod canon_log2 {
    /// The nonlinear value of scene-linear 0.0.
    pub const NONLINEAR_BLACK: f32 = 0.092864126;

    /// The scene-linear value of nonlinear value 0.0.
    pub const LINEAR_MIN: f32 = -0.016363228;

    /// The scene-linear value of nonlinear value 1.0.
    pub const LINEAR_MAX: f32 = 65.816086;

    const A: f32 = 0.24136077;
    const B: f32 = 87.099375;
    const C: f32 = 0.092864125;

    /// Linear -> Canon Log 2
    pub fn from_linear(x: f32) -> f32 {
        if x < 0.0 {
            -A * (1.0 - (B * x)).log10() + C
        } else {
            A * (1.0 + (B * x)).log10() + C
        }
    }

    /// Canon Log 2 -> Linear
    pub fn to_linear(x: f32) -> f32 {
        if x < C {
            -(10.0f32.powf((C - x) / A) - 1.0) / B
        } else {
            (10.0f32.powf((x - C) / A) - 1.0) / B
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            // Invariants from page 9 of "Canon Log Gamma Curves -
            // Description of the Canon Log, Canon Log 2 and Canon Log 3
            // Gamma Curves", from Canon, November 1st 2018.
            assert!((from_linear(0.0) - 0.093).abs() < 0.001);
            assert!((from_linear(0.2) - 0.398).abs() < 0.001);
            assert!((from_linear(1.0) - 0.562).abs() < 0.001);
            assert!((from_linear(8.0) - 0.779).abs() < 0.001);
            assert!((from_linear(16.0) - 0.852).abs() < 0.001);
            assert!((from_linear(64.0) - 0.997).abs() < 0.001);
        }

        #[test]
        fn to_linear_test() {
            // Invariants from page 9 of "Canon Log Gamma Curves -
            // Description of the Canon Log, Canon Log 2 and Canon Log 3
            // Gamma Curves", from Canon, November 1st 2018.
            assert!((to_linear(0.093) - 0.0).abs() < 0.001);
            assert!((to_linear(0.398) - 0.2).abs() < 0.001);
            assert!((to_linear(0.562) - 1.0).abs() < 0.003);
            assert!((to_linear(0.779) - 8.0).abs() < 0.02);
            assert!((to_linear(0.852) - 16.0).abs() < 0.03);
            assert!((to_linear(0.997) - 64.0).abs() < 0.05);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = (i as f32 / 1023.0) * (LINEAR_MAX - LINEAR_MIN) + LINEAR_MIN;
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_1);
            }
        }
    }
}

/// Canon Log 3.
///
/// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
/// mapping.  It is a transfer function between "scene linear" and a
/// nonlinear [0.0, 1.0] range.  For example, scene-linear 0.0 maps to
/// `NONLINEAR_BLACK` (which is > 0.0), and a nonlinear value of 1.0 maps
/// to a much greater than 1.0 scene-linear value.
pub mod canon_log3 {
    /// The nonlinear value of scene-linear 0.0.
    pub const NONLINEAR_BLACK: f32 = 0.12512219;

    /// The scene-linear value of nonlinear value 0.0.
    pub const LINEAR_MIN: f32 = -0.08201483;

    /// The scene-linear value of nonlinear value 1.0.
    pub const LINEAR_MAX: f32 = 16.298117;

    const A: f32 = 14.98325;
    const B: f32 = 1.9754798;
    const C: f32 = 0.36726845;
    const D: f32 = 0.12783901;
    const E: f32 = 0.12512219;
    const F: f32 = 0.12240537;

    /// Linear -> Canon Log 3
    pub fn from_linear(x: f32) -> f32 {
        const BOUND: f32 = 0.014;
        if x < -BOUND {
            -C * (1.0 - (A * x)).log10() + D
        } else if x <= BOUND {
            (B * x) + E
        } else {
            C * (1.0 + (A * x)).log10() + F
        }
    }

    /// Canon Log 3 -> Linear
    pub fn to_linear(x: f32) -> f32 {
        const BOUND1: f32 = 0.097465473;
        const BOUND2: f32 = 0.15277891;
        if x < BOUND1 {
            -(10.0f32.powf((D - x) / C) - 1.0) / A
        } else if x <= BOUND2 {
            (x - E) / B
        } else {
            (10.0f32.powf((x - F) / C) - 1.0) / A
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            // Invariants from page 9 of "Canon Log Gamma Curves -
            // Description of the Canon Log, Canon Log 2 and Canon Log 3
            // Gamma Curves", from Canon, November 1st 2018.
            assert!((from_linear(0.0) - 0.125).abs() < 0.001);
            assert!((from_linear(0.2) - 0.343).abs() < 0.001);
            assert!((from_linear(1.0) - 0.564).abs() < 0.001);
            assert!((from_linear(8.0) - 0.887).abs() < 0.001);
            assert!((from_linear(16.0) - 0.997).abs() < 0.001);
        }

        #[test]
        fn to_linear_test() {
            // Invariants from page 9 of "Canon Log Gamma Curves -
            // Description of the Canon Log, Canon Log 2 and Canon Log 3
            // Gamma Curves", from Canon, November 1st 2018.
            assert!((to_linear(0.125) - 0.0).abs() < 0.001);
            assert!((to_linear(0.343) - 0.2).abs() < 0.001);
            assert!((to_linear(0.564) - 1.0).abs() < 0.004);
            assert!((to_linear(0.887) - 8.0).abs() < 0.01);
            assert!((to_linear(0.997) - 16.0).abs() < 0.01);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = (i as f32 / 1023.0) * (LINEAR_MAX - LINEAR_MIN) + LINEAR_MIN;
                assert!((n - to_linear(from_linear(n))).abs() < 0.000_01);
            }
        }
    }
}
