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
                assert!((n - from_linear(to_linear(n))).abs() < 0.000_001);
            }
        }
    }
}

/// Perceptual Quantizer from Rec.2100.
///
/// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
/// mapping.  It is a transfer function between linear
/// [0.0, `LUMINANCE_MAX`] (in cd/m^2) and non-linear [0.0, 1.0].
pub mod rec2100_pq {
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
        // Hack so the function is well defined below 0.0.
        let flip = n < 0.0;
        let n = n.abs();

        // The actual transfer function.
        let n = n * (1.0 / LUMINANCE_MAX);
        let n_m1 = n.powf(M1);
        let out = ((C1 + (C2 * n_m1)) / (1.0 + (C3 * n_m1))).powf(M2);

        // Hack again.
        if flip {
            out * -1.0
        } else {
            out
        }
    }

    /// PQ -> Linear.
    ///
    /// Input is in the range [0.0, 1.0].
    /// Output is in the range [0, `LUMINANCE_MAX`], representing display
    /// luminance in cd/m^2.
    #[inline(always)]
    pub fn to_linear(n: f32) -> f32 {
        // Hack so the function is well defined below 0.0.
        let flip = n < 0.0;
        let n = n.abs();

        // The actual transfer function.
        let n_1_m2 = n.powf(1.0 / M2);
        let linear = ((n_1_m2 - C1).max(0.0) / (C2 - (C3 * n_1_m2))).powf(1.0 / M1);
        let out = linear * LUMINANCE_MAX;

        // Hack again.
        if flip {
            out * -1.0
        } else {
            out
        }
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
                assert!((n - from_linear(to_linear(n))).abs() < 0.000_1);
            }
        }
    }
}

/// Hybrid Log-Gamma from Rec.2100.
pub mod rec2100_hlg {
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
                assert!((n - from_linear(to_linear(n))).abs() < 0.000_001);
            }
        }
    }
}

/// Arri's ALEXA Log C V3 transfer function family.
///
/// Unlike the other transfer functions in Colorbox, the ALEXA Log C
/// transfer function is parameterized, representing a family of
/// of transfer functions.  Specifically, they are parameterized by
/// two things:
///
/// - `is_ev`: whether you're converting to/from exposure values (`true`)
///   or raw sensor signal (`false`).  This will typically be `true`:
///   since raw sensor signal data from ALEXA cameras is already linear,
///   cases where you would want this to be `false` are pretty niche.
/// - `exposure_index`: the exposure index (or "EI") the footage was shot
///   with.  This information is included in the metadata of Arri ALEXA
///   footage files.
///
/// Note: it's possible for footage to have an exposure index greater
/// than 1600, which is the maximim that the functions in this module
/// support.  Unfortunately, Arri does not document analytic transfer
/// functions for exposure indices greater than 1600.  For such footage,
/// Arri provides lookup tables for download.
///
/// For more details, see Arri's white paper "ALEXA LogC Curve - Usage in VFX".
pub mod alexa_logc {
    // /// The nonlinear value of scene-linear 0.0.
    // pub const NONLINEAR_BLACK: f32 = 0.12512247;

    // /// The scene-linear value of nonlinear value 0.0.
    // pub const LINEAR_MIN: f32 = -0.087466866;

    // /// The scene-linear value of nonlinear value 1.0.
    // pub const LINEAR_MAX: f32 = 8.295911;

    /// Exposure index.
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub enum EI {
        Ei160,
        Ei200,
        Ei250,
        Ei320,
        Ei400,
        Ei500,
        Ei640,
        Ei800,
        Ei1000,
        Ei1280,
        Ei1600,
    }

    /// Linear -> Log.
    pub fn from_linear(x: f32, is_ev: bool, exposure_index: EI) -> f32 {
        let [cut, a, b, c, d, e, f] = if is_ev {
            ei_ev(exposure_index)
        } else {
            ei_sensor(exposure_index)
        };

        if x < cut {
            e * x + f
        } else {
            c * (a * x + b).log10() + d
        }
    }

    /// Log -> Linear.
    pub fn to_linear(x: f32, is_ev: bool, exposure_index: EI) -> f32 {
        let [cut, a, b, c, d, e, f] = if is_ev {
            ei_ev(exposure_index)
        } else {
            ei_sensor(exposure_index)
        };

        if x < (e * cut + f) {
            (x - f) / e
        } else {
            (10.0f32.powf((x - d) / c) - b) / a
        }
    }

    //---------------------------------------------------------

    type EIValues = [f32; 7];

    // Function parameters for converting between sensor signal and Log C, at various exposure indices.
    fn ei_sensor(ei: EI) -> EIValues {
        match ei {
            EI::Ei160 => [
                0.004680, 40.0, -0.076072, 0.269036, 0.381991, 42.062665, -0.071569,
            ],
            EI::Ei200 => [
                0.004597, 50.0, -0.118740, 0.266007, 0.382478, 51.986387, -0.110339,
            ],
            EI::Ei250 => [
                0.004518, 62.5, -0.171260, 0.262978, 0.382966, 64.243053, -0.158224,
            ],
            EI::Ei320 => [
                0.004436, 80.0, -0.243808, 0.259627, 0.383508, 81.183335, -0.224409,
            ],
            EI::Ei400 => [
                0.004369, 100.0, -0.325820, 0.256598, 0.383999, 100.295280, -0.299079,
            ],
            EI::Ei500 => [
                0.004309, 125.0, -0.427461, 0.253569, 0.384493, 123.889239, -0.391261,
            ],
            EI::Ei640 => [
                0.004249, 160.0, -0.568709, 0.250219, 0.385040, 156.482680, -0.518605,
            ],
            EI::Ei800 => [
                0.004201, 200.0, -0.729169, 0.247190, 0.385537, 193.235573, -0.662201,
            ],
            EI::Ei1000 => [
                0.004160, 250.0, -0.928805, 0.244161, 0.386036, 238.584745, -0.839385,
            ],
            EI::Ei1280 => [
                0.004120, 320.0, -1.207168, 0.240810, 0.386590, 301.197380, -1.084020,
            ],
            EI::Ei1600 => [
                0.004088, 400.0, -1.524256, 0.237781, 0.387093, 371.761171, -1.359723,
            ],
        }
    }

    // Function parameters for converting between exposure value and Log C, at various exposure indices.
    fn ei_ev(ei: EI) -> EIValues {
        match ei {
            EI::Ei160 => [
                0.005561, 5.555556, 0.080216, 0.269036, 0.381991, 5.842037, 0.092778,
            ],
            EI::Ei200 => [
                0.006208, 5.555556, 0.076621, 0.266007, 0.382478, 5.776265, 0.092782,
            ],
            EI::Ei250 => [
                0.006871, 5.555556, 0.072941, 0.262978, 0.382966, 5.710494, 0.092786,
            ],
            EI::Ei320 => [
                0.007622, 5.555556, 0.068768, 0.259627, 0.383508, 5.637732, 0.092791,
            ],
            EI::Ei400 => [
                0.008318, 5.555556, 0.064901, 0.256598, 0.383999, 5.571960, 0.092795,
            ],
            EI::Ei500 => [
                0.009031, 5.555556, 0.060939, 0.253569, 0.384493, 5.506188, 0.092800,
            ],
            EI::Ei640 => [
                0.009840, 5.555556, 0.056443, 0.250219, 0.385040, 5.433426, 0.092805,
            ],
            EI::Ei800 => [
                0.010591, 5.555556, 0.052272, 0.247190, 0.385537, 5.367655, 0.092809,
            ],
            EI::Ei1000 => [
                0.011361, 5.555556, 0.047996, 0.244161, 0.386036, 5.301883, 0.092814,
            ],
            EI::Ei1280 => [
                0.012235, 5.555556, 0.043137, 0.240810, 0.386590, 5.229121, 0.092819,
            ],
            EI::Ei1600 => [
                0.013047, 5.555556, 0.038625, 0.237781, 0.387093, 5.163350, 0.092824,
            ],
        }
    }

    //---------------------------------------------------------

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = i as f32 / 1023.0;
                assert!(
                    (n - from_linear(to_linear(n, true, EI::Ei800), true, EI::Ei800)).abs()
                        < 0.000_001
                );
            }
        }
    }
}

pub mod blackmagic;

/// Canon's transfer functions.
pub mod canon {
    /// Canon Log (original).
    ///
    /// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
    /// mapping.  It is a transfer function between "scene linear" and a
    /// nonlinear [0.0, 1.0] range.  For example, scene-linear 0.0 maps to
    /// `NONLINEAR_BLACK` (which is > 0.0), and a nonlinear value of 1.0 maps
    /// to a much greater than 1.0 scene-linear value.
    pub mod log1 {
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
            fn constants() {
                assert_eq!(from_linear(0.0), NONLINEAR_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

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
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_01);
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
    pub mod log2 {
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
            fn constants() {
                assert_eq!(from_linear(0.0), NONLINEAR_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

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
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_1);
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
    pub mod log3 {
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
            fn constants() {
                assert_eq!(from_linear(0.0), NONLINEAR_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

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
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_01);
                }
            }
        }
    }
}

/// DJI's transfer function.
pub mod dji {
    /// DJI's D-Log.
    ///
    /// Note that according to the D-Log white paper:
    ///
    /// > For EI above 1600, the output code value will be beyond saturation
    /// > level if calculated by the linear-log conversion function, thus an
    /// > s-shape function is applied to keep output code value in valid
    /// > range.
    ///
    /// The s-curve used and the precise way in which it is applied are
    /// undocumented, and therefore no attempt is made in this code to
    /// account for it.  The functions in this module just implement vanilla
    /// D-Log.
    ///
    /// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
    /// mapping.  It is a transfer function between "scene linear" and
    /// normalized "code values".  For example, scene-linear 0.0
    /// maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
    /// 1.0 maps to a much greater than 1.0 scene linear value.
    pub mod dlog {
        /// The normalized code value of scene-linear 0.0.
        pub const CV_BLACK: f32 = 0.0929;

        /// The scene-linear value of normalized code value 0.0.
        pub const LINEAR_MIN: f32 = -0.015419087;

        /// The scene-linear value of normalized code value 1.0.
        pub const LINEAR_MAX: f32 = 41.999413;

        const CUT_1: f32 = 0.0078;
        const CUT_2: f32 = 0.14;
        const A: f32 = 0.9892;
        const B: f32 = 0.0108;
        const C: f32 = 0.256663;
        const D: f32 = 0.584555;
        const E: f32 = 6.025;
        const F: f32 = 0.0929;

        /// From scene linear to (normalized) code values.
        ///
        /// For example, to get 10-bit code values do
        /// `from_linear(scene_linear_in) * 1023.0`
        #[inline]
        pub fn from_linear(x: f32) -> f32 {
            if x < CUT_1 {
                E * x + F
            } else {
                C * (A * x + B).log10() + D
            }
        }

        /// From (normalized) code values to scene linear.
        ///
        /// For example, if using 10-bit code values do
        /// `to_linear(10_bit_cv_in / 1023.0)`
        #[inline]
        pub fn to_linear(x: f32) -> f32 {
            if x < CUT_2 {
                (x - F) / E
            } else {
                (10.0f32.powf((x - D) / C) - B) / A
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn constants() {
                assert_eq!(from_linear(0.0), CV_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

            #[test]
            fn from_linear_test() {
                // Invariants from page 3 of "White Paper on D-Log and
                // D-Gamut" Revision 1.0, from DJI, September 29th, 2017.
                assert!((from_linear(0.0) - (95.0 / 1023.0)).abs() < 0.001);
                assert!((from_linear(0.18) - (408.0 / 1023.0)).abs() < 0.001);
                assert!((from_linear(0.9) - (586.0 / 1023.0)).abs() < 0.001);
            }

            #[test]
            fn to_linear_test() {
                // Invariants from page 3 of "White Paper on D-Log and
                // D-Gamut" Revision 1.0, from DJI, September 29th, 2017.
                assert!((to_linear(95.0 / 1023.0) - 0.0).abs() < 0.001);
                assert!((to_linear(408.0 / 1023.0) - 0.18).abs() < 0.001);
                assert!((to_linear(586.0 / 1023.0) - 0.9).abs() < 0.03);
            }

            #[test]
            fn round_trip() {
                for i in 0..1024 {
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_001);
                }
            }
        }
    }
}

/// Fujifilm's transfer function.
pub mod fujifilm {
    /// Fujifilm's F-Log.
    ///
    /// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
    /// mapping.  It is a transfer function between "scene linear" and
    /// normalized "code values".  For example, scene-linear 0.0
    /// maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
    /// 1.0 maps to a much greater than 1.0 scene linear value.
    pub mod flog {
        /// The normalized code value of scene-linear 0.0.
        pub const CV_BLACK: f32 = 0.092864;

        /// The scene-linear value of normalized code value 0.0.
        pub const LINEAR_MIN: f32 = -0.010630486;

        /// The scene-linear value of normalized code value 1.0.
        pub const LINEAR_MAX: f32 = 7.281325;

        const CUT_1: f32 = 0.00089;
        const CUT_2: f32 = 0.100_537_775_223_865;
        const A: f32 = 0.555556;
        const B: f32 = 0.009468;
        const C: f32 = 0.344676;
        const D: f32 = 0.790453;
        const E: f32 = 8.735631;
        const F: f32 = 0.092864;

        /// From scene linear to (normalized) code values.
        ///
        /// For example, to get 10-bit code values do
        /// `from_linear(scene_linear_in) * 1023.0`
        #[inline]
        pub fn from_linear(x: f32) -> f32 {
            if x < CUT_1 {
                E * x + F
            } else {
                C * (A * x + B).log10() + D
            }
        }

        /// From (normalized) code values to scene linear.
        ///
        /// For example, if using 10-bit code values do
        /// `to_linear(10_bit_cv_in / 1023.0)`
        #[inline]
        pub fn to_linear(x: f32) -> f32 {
            if x < CUT_2 {
                (x - F) / E
            } else {
                (10.0f32.powf((x - D) / C) - B) / A
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn constants() {
                assert_eq!(from_linear(0.0), CV_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

            #[test]
            fn from_linear_test() {
                // Invariants from page 2 of "F-Log Data Sheet Ver. 1.0"
                // from Fujifilm.
                assert!((from_linear(0.0) - (95.0 / 1023.0)).abs() < 0.001);
                assert!((from_linear(0.18) - (470.0 / 1023.0)).abs() < 0.001);
                assert!((from_linear(0.9) - (705.0 / 1023.0)).abs() < 0.001);
            }

            #[test]
            fn to_linear_test() {
                // Invariants from page 2 of "F-Log Data Sheet Ver. 1.0"
                // from Fujifilm.
                assert!((to_linear(95.0 / 1023.0) - 0.0).abs() < 0.001);
                assert!((to_linear(470.0 / 1023.0) - 0.18).abs() < 0.001);
                assert!((to_linear(705.0 / 1023.0) - 0.9).abs() < 0.03);
            }

            #[test]
            fn round_trip() {
                for i in 0..1024 {
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_001);
                }
            }
        }
    }
}

/// Nikon's transfer function.
pub mod nikon {
    /// Nikon's N-Log.
    ///
    /// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
    /// mapping.  It is a transfer function between "scene linear" and
    /// normalized "code values".  For example, scene-linear 0.0
    /// maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
    /// 1.0 maps to a much greater than 1.0 scene linear value.
    pub mod nlog {
        /// The normalized code value of scene-linear 0.0.
        pub const CV_BLACK: f32 = 0.12437262;

        /// The scene-linear value of normalized code value 0.0.
        pub const LINEAR_MIN: f32 = -0.0075;

        /// The scene-linear value of normalized code value 1.0.
        pub const LINEAR_MAX: f32 = 14.780865;

        // The `CUT_1` and `CUT_2` constants are slightly different
        // than in the Nikon white paper, because the official constants
        // are only precise enough to connect the piece-wise curves in
        // 10-bit color.  These constants are much higher precision, and
        // were derived to connect the piece-wise curves properly even
        // in much higher precision color.
        const CUT_1: f32 = 0.316731;
        const CUT_2: f32 = 0.436505;
        const A: f32 = 650.0 / 1023.0;
        const B: f32 = 0.0075;
        const C: f32 = 150.0 / 1023.0;
        const D: f32 = 619.0 / 1023.0;

        /// From scene linear to (normalized) code values.
        ///
        /// For example, to get 10-bit code values do
        /// `from_linear(scene_linear_in) * 1023.0`
        #[inline]
        pub fn from_linear(x: f32) -> f32 {
            if x < CUT_1 {
                A * (x + B).powf(1.0 / 3.0)
            } else {
                C * x.ln() + D
            }
        }

        /// From (normalized) code values to scene linear.
        ///
        /// For example, if using 10-bit code values do
        /// `to_linear(10_bit_cv_in / 1023.0)`
        #[inline]
        pub fn to_linear(x: f32) -> f32 {
            if x < CUT_2 {
                let tmp = x / A;
                tmp * tmp * tmp - B
            } else {
                ((x - D) / C).exp()
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn constants() {
                assert_eq!(from_linear(0.0), CV_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

            // The Nikon white paper specifies the formula in terms of
            // 10-bit code values.  These are a straight implementation
            // of their exact formulas to verify against.
            fn from_linear_10bit(x: f32) -> f32 {
                if x < 0.328 {
                    650.0 * (x + 0.0075).powf(1.0 / 3.0)
                } else {
                    150.0 * x.ln() + 619.0
                }
            }
            fn to_linear_10bit(x: f32) -> f32 {
                if x < 452.0 {
                    let tmp = x / 650.0;
                    tmp * tmp * tmp - 0.0075
                } else {
                    ((x - 619.0) / 150.0).exp()
                }
            }

            // Make sure it matches the official formulas after
            // normalization.
            #[test]
            fn matches_10_bit() {
                for i in 0..1024 {
                    let n = i as f32;

                    let l1 = to_linear(n / 1023.0);
                    let l2 = to_linear_10bit(n);
                    assert!((1.0 - (l1 / l2)).abs() < 0.001);

                    let cv1 = from_linear(l1);
                    let cv2 = from_linear_10bit(l1);
                    assert!((cv1 - (cv2 / 1023.0)).abs() < 0.001);
                }
            }

            #[test]
            fn from_linear_test() {
                // The paper "N-Log Specification Document" version 1.0.0 (by
                // Nikon, September 1st, 2018) does not provide any tables of
                // known inputs/outputs to verify against.  So instead these
                // test cases were built by making sure they roughly matched
                // the visual graph on page 4 of the document.
                assert!((from_linear(0.0) - (128.0 / 1023.0)).abs() < 0.001);
                assert!((from_linear(0.18) - (372.0 / 1023.0)).abs() < 0.001);
                assert!((from_linear(0.9) - (603.0 / 1023.0)).abs() < 0.001);
            }

            #[test]
            fn to_linear_test() {
                // The paper "N-Log Specification Document" version 1.0.0 (by
                // Nikon, September 1st, 2018) does not provide any tables of
                // known inputs/outputs to verify against.  So instead these
                // test cases were built by making sure they roughly matched
                // the visual graph on page 4 of the document.
                assert!((to_linear(128.0 / 1023.0) - 0.0).abs() < 0.001);
                assert!((to_linear(372.0 / 1023.0) - 0.18).abs() < 0.001);
                assert!((to_linear(603.0 / 1023.0) - 0.9).abs() < 0.002);
            }

            #[test]
            fn round_trip() {
                for i in 0..1024 {
                    let n = i as f32 / 1023.0;
                    let n2 = from_linear(to_linear(n));
                    assert!((n - n2).abs() < 0.000_01);
                }
            }
        }
    }
}

/// Panasonic's transfer function.
pub mod panasonic {
    /// Panasonic's V-Log.
    ///
    /// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
    /// mapping.  It is a transfer function between "scene linear" and
    /// normalized "code values".  For example, scene-linear 0.0
    /// maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
    /// 1.0 maps to a much greater than 1.0 scene linear value.
    pub mod vlog {
        /// The normalized code value of scene-linear 0.0.
        pub const CV_BLACK: f32 = 0.125;

        /// The scene-linear value of normalized code value 0.0.
        pub const LINEAR_MIN: f32 = -0.02232143;

        /// The scene-linear value of normalized code value 1.0.
        pub const LINEAR_MAX: f32 = 46.085537;

        const CUT_1: f32 = 0.01;
        const CUT_2: f32 = 0.181;
        const B: f32 = 0.00873;
        const C: f32 = 0.241514;
        const D: f32 = 0.598206;

        /// From scene linear to (normalized) code values.
        ///
        /// For example, to get 10-bit code values do
        /// `from_linear(scene_linear_in) * 1023.0`
        #[inline]
        pub fn from_linear(x: f32) -> f32 {
            if x < CUT_1 {
                5.6 * x + 0.125
            } else {
                C * (x + B).log10() + D
            }
        }

        /// From (normalized) code values to scene linear.
        ///
        /// For example, if using 10-bit code values do
        /// `to_linear(10_bit_cv_in / 1023.0)`
        #[inline]
        pub fn to_linear(x: f32) -> f32 {
            if x < CUT_2 {
                (x - 0.125) / 5.6
            } else {
                10.0f32.powf((x - D) / C) - B
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn constants() {
                assert_eq!(from_linear(0.0), CV_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

            #[test]
            fn from_linear_test() {
                // Invariants from page 3 of "V-Log/V-Gamut Reference Manual"
                // from Panasonic, November 28th 2014.
                assert!((from_linear(0.0) - (128.0 / 1023.0)).abs() < 0.001);
                assert!((from_linear(0.18) - (433.0 / 1023.0)).abs() < 0.001);
                assert!((from_linear(0.9) - (602.0 / 1023.0)).abs() < 0.001);
            }

            #[test]
            fn to_linear_test() {
                // Invariants from page 3 of "V-Log/V-Gamut Reference Manual"
                // from Panasonic, November 28th 2014.
                assert!((to_linear(128.0 / 1023.0) - 0.0).abs() < 0.001);
                assert!((to_linear(433.0 / 1023.0) - 0.18).abs() < 0.001);
                assert!((to_linear(602.0 / 1023.0) - 0.9).abs() < 0.03);
            }

            #[test]
            fn round_trip() {
                for i in 0..1024 {
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_001);
                }
            }
        }
    }
}

/// RED's transfer function.
pub mod red {
    /// RED's Log3G10.
    ///
    /// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
    /// mapping.  It is a transfer function between "scene linear" and
    /// normalized "code values".  For example, scene-linear 0.0
    /// maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
    /// 1.0 maps to a much greater than 1.0 scene linear value.
    pub mod log3g10 {
        /// The normalized code value of scene-linear 0.0.
        pub const CV_BLACK: f32 = 0.09155148;

        /// The scene-linear value of normalized code value 0.0.
        pub const LINEAR_MIN: f32 = -0.01;

        /// The scene-linear value of normalized code value 1.0.
        pub const LINEAR_MAX: f32 = 184.32233;

        const A: f32 = 0.224282;
        const B: f32 = 155.975327;
        const C: f32 = 0.01;
        const G: f32 = 15.1927;

        /// From scene linear to (normalized) code values.
        ///
        /// For example, to get 10-bit code values do
        /// `from_linear(scene_linear_in) * 1023.0`
        #[inline]
        pub fn from_linear(x: f32) -> f32 {
            let x = x + C;

            if x < 0.0 {
                x * G
            } else {
                A * ((x * B) + 1.0).log10()
            }
        }

        /// From (normalized) code values to scene linear.
        ///
        /// For example, if using 10-bit code values do
        /// `to_linear(10_bit_cv_in / 1023.0)`
        #[inline]
        pub fn to_linear(x: f32) -> f32 {
            if x < 0.0 {
                (x / G) - C
            } else {
                ((10.0f32.powf(x / A) - 1.0) / B) - C
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn constants() {
                assert_eq!(from_linear(0.0), CV_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

            #[test]
            fn from_linear_test() {
                // Invariants from page 5 of "White Paper on WedWideGamutRGB and Log3G10" from RED.
                assert!((from_linear(-0.01) - 0.0).abs() < 0.00001);
                assert!((from_linear(0.0) - 0.091551).abs() < 0.00001);
                assert!((from_linear(0.18) - 0.333333).abs() < 0.00001);
                assert!((from_linear(1.0) - 0.493449).abs() < 0.00001);
                assert!((from_linear(184.322) - 1.0).abs() < 0.00001);
            }

            #[test]
            fn to_linear_test() {
                // Invariants from page 5 of "White Paper on WedWideGamutRGB and Log3G10" from RED.
                assert!((to_linear(0.0) - -0.01).abs() < 0.00001);
                assert!((to_linear(0.091551) - 0.0).abs() < 0.00001);
                assert!((to_linear(0.333333) - 0.18).abs() < 0.00001);
                assert!((to_linear(0.493449) - 1.0).abs() < 0.00001);
                assert!((to_linear(1.0) - 184.322).abs() < 0.001);
            }

            #[test]
            fn round_trip() {
                for i in 0..1024 {
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_001);
                }
            }
        }
    }
}

/// Sony's transfer functions.
pub mod sony {
    /// Sony's S-Log (original).
    ///
    /// Note: this transfer function is not a [0.0, 1.0] -> [0.0, 1.0]
    /// mapping.  It is a transfer function between "scene linear" and
    /// normalized "code values".  For example, scene-linear 0.0
    /// maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
    /// 1.0 maps to a much greater than 1.0 scene linear value.
    pub mod slog1 {
        /// The normalized code value of scene-linear 0.0.
        pub const CV_BLACK: f32 = 0.088251315;

        /// The normalized code value of camera sensor saturation.
        pub const CV_SATURATION: f32 = SLOG_WHITE;

        /// The scene-linear value of normalized code value 0.0.
        pub const LINEAR_MIN: f32 = -0.014279289;

        /// The scene-linear value of normalized code value 1.0.
        pub const LINEAR_MAX: f32 = 9.737593;

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
            fn constants() {
                assert_eq!(from_linear(0.0), CV_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

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
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_001);
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
    pub mod slog2 {
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
            fn constants() {
                assert_eq!(from_linear(0.0), CV_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

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
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_001);
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
    pub mod slog3 {
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
            fn constants() {
                assert_eq!(from_linear(0.0), CV_BLACK);
                assert_eq!(to_linear(0.0), LINEAR_MIN);
                assert_eq!(to_linear(1.0), LINEAR_MAX);
            }

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
                    let n = i as f32 / 1023.0;
                    assert!((n - from_linear(to_linear(n))).abs() < 0.000_001);
                }
            }
        }
    }
}
