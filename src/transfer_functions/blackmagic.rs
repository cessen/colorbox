//! Blackmagic Design's transfer functions.
//!
//! Unfortunately, Blackmagic Design does not publish most of their
//! transfer functions, so with the notable exceptions of "Film Generation 5"
//! and "DaVinci Intermediate", all the transfer functions in this module are
//! reverse engineered, not official.  They are neverthless more than accurate
//! enough for any practical purpose.  See the following page for details:
//!
//! <https://psychopath.io/post/2022_04_23_blackmagic_design_color_spaces>
//!
//! That also means these are not all of Blackmagic Design's transfer
//! functions, only the two that they published and the remaining that
//! have been reverse engineered so far.
//!
//! Note: none of the transfer functions in this module are
//! [0.0, 1.0] -> [0.0, 1.0] mappings.  They are transfer functions between
//! "scene linear" and normalized "code values".  For example, scene-linear 0.0
//! maps to `CV_BLACK` (which is > 0.0), and a normalized code value of
//! 1.0 maps to a much greater than 1.0 scene linear value.

macro_rules! bmd_log_tf {
    (
        $a:literal,
        $b:literal,
        $c:literal,
        $d:literal,
        $e:literal,
        $lin_cut:literal,
        $cv_black:literal,
        $linear_min:literal,
        $linear_max:literal
        $(,)?
    ) => {
        /// The normalized code value of scene-linear 0.0.
        pub const CV_BLACK: f32 = $cv_black;

        /// The scene-linear value of normalized code value 0.0.
        pub const LINEAR_MIN: f32 = $linear_min;

        /// The scene-linear value of normalized code value 1.0.
        pub const LINEAR_MAX: f32 = $linear_max;

        const A: f32 = $a;
        const B: f32 = $b;
        const C: f32 = $c;
        const D: f32 = $d;
        const E: f32 = $e;

        const LIN_CUT: f32 = $lin_cut;
        const LOG_CUT: f32 = LIN_CUT * A + B;

        /// From scene linear to (normalized) code values.
        #[inline]
        pub fn from_linear(x: f32) -> f32 {
            if x < LIN_CUT {
                x * A + B
            } else {
                (x + C).ln() * D + E
            }
        }

        /// From (normalized) code values to scene linear.
        #[inline]
        pub fn to_linear(x: f32) -> f32 {
            if x < LOG_CUT {
                (x - B) / A
            } else {
                ((x - E) / D).exp() - C
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
            fn round_trip() {
                for i in 0..1024 {
                    let n = (i as f32 / 1023.0) * (LINEAR_MAX - LINEAR_MIN) + LINEAR_MIN;
                    assert!(((n - to_linear(from_linear(n))).abs() / n.abs()) < 0.000_01);
                }
            }
        }
    }
}

/// Blackmagic Design's "Film Generation 5".
///
/// The Blackmagic Design whitepaper for this transfer function
/// specifies both a "Film Generation 5" value and a "10-bit video
/// levels" value, the latter of which uses broadcast legal ranges
/// and the former of which doesn't.  That puts this implementation
/// in a bit of a bind, because the normalized encoded values of
/// those two approaches are different.
///
/// Since the "Film Generation 5" values are simpler in nature
/// and directly reflect the functions given in the whitepaper,
/// the functions within this module encode/decode for that.
/// If you want encoded values with legal ranges, it's a linear
/// mapping from the "Film Generation 5" values.  From Gen 5 to
/// 10-bit legal ranges:
///
/// - `0.0924657534246575 -> 145`
/// - `1.0 -> 940`
pub mod film_gen5 {
    /// The normalized code value of scene-linear 0.0.
    pub const CV_BLACK: f32 = 0.09246575;

    /// The scene-linear value of normalized code value 0.0.
    pub const LINEAR_MIN: f32 = -0.011162501;

    /// The scene-linear value of normalized code value 1.0.
    pub const LINEAR_MAX: f32 = 222.86098;

    const A: f32 = 8.283605932402494;
    const B: f32 = 0.09246575342465753;
    const C: f32 = 0.005494072432257808;
    const D: f32 = 0.08692876065491224;
    const E: f32 = 0.5300133392291939;
    const LIN_CUT: f32 = 0.005;
    const LOG_CUT: f32 = LIN_CUT * A + B;

    /// From scene linear to (normalized) code values.
    ///
    /// For example, to get 10-bit code values do
    /// `from_linear(scene_linear_in) * 1023.0`
    #[inline]
    pub fn from_linear(x: f32) -> f32 {
        if x < LIN_CUT {
            x * A + B
        } else {
            (x + C).ln() * D + E
        }
    }

    /// From (normalized) code values to scene linear.
    ///
    /// For example, if using 10-bit code values do
    /// `to_linear(10_bit_cv_in / 1023.0)`
    #[inline]
    pub fn to_linear(x: f32) -> f32 {
        if x < LOG_CUT {
            (x - B) / A
        } else {
            ((x - E) / D).exp() - C
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_linear_test() {
            // Invariants from page 3 of "Blackmagic Generation 5 Color Science"
            // from Blackmagic Design, May 2021.
            assert!((from_linear(0.0) - 0.0924657534246575).abs() < 0.00001);
            assert!((from_linear(0.18) - 0.3835616438356165).abs() < 0.00001);
            assert!((from_linear(1.0) - 0.5304896249573048).abs() < 0.00001);
            assert!((from_linear(10.0) - 0.7302219538415439).abs() < 0.00001);
            assert!((from_linear(40.0) - 0.8506949973834717).abs() < 0.00001);
            assert!((from_linear(100.0) - 0.9303398518999735).abs() < 0.00001);
            assert!((from_linear(222.86) - 1.0).abs() < 0.00001);
        }

        #[test]
        fn to_linear_test() {
            // Invariants from page 3 of "Blackmagic Generation 5 Color Science"
            // from Blackmagic Design, May 2021.
            assert!((to_linear(0.0924657534246575) - 0.0).abs() < 0.00001);
            assert!((to_linear(0.3835616438356165) - 0.18).abs() < 0.00001);
            assert!((to_linear(0.5304896249573048) - 1.0).abs() < 0.00001);
            assert!((to_linear(0.7302219538415439) - 10.0).abs() < 0.00001);
            assert!((to_linear(0.8506949973834717) - 40.0).abs() < 0.0001);
            assert!((to_linear(0.9303398518999735) - 100.0).abs() < 0.0001);
            assert!((to_linear(1.0) - 222.86).abs() < 0.001);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = (i as f32 / 1023.0) * (LINEAR_MAX - LINEAR_MIN) + LINEAR_MIN;
                assert!(((n - to_linear(from_linear(n))).abs() / n.abs()) < 0.000_001);
            }
        }
    }
}

/// Blackmagic Design's "DaVinci Intermediate".
pub mod davinci_intermediate {
    /// The normalized code value of scene-linear 0.0.
    pub const CV_BLACK: f32 = 0.0;

    /// The scene-linear value of normalized code value 0.0.
    pub const LINEAR_MIN: f32 = 0.0;

    /// The scene-linear value of normalized code value 1.0.
    pub const LINEAR_MAX: f32 = 100.00002;

    const A: f32 = 0.0075;
    const B: f32 = 7.0;
    const C: f32 = 0.07329248;
    const M: f32 = 10.44426855;
    const LIN_CUT: f32 = 0.00262409;
    const LOG_CUT: f32 = LIN_CUT * M;

    /// From scene linear to (normalized) code values.
    #[inline]
    pub fn from_linear(x: f32) -> f32 {
        if x < LIN_CUT {
            x * M
        } else {
            ((x + A).log2() + B) * C
        }
    }

    /// From (normalized) code values to scene linear.
    #[inline]
    pub fn to_linear(x: f32) -> f32 {
        if x < LOG_CUT {
            x / M
        } else {
            2.0f32.powf((x / C) - B) - A
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
            // Invariants from page 4 of "Wide Gamut Intermediat - DaVinci Resolve 17"
            // from Blackmagic Design, August 2021.
            assert!((from_linear(-0.01) - -0.104443).abs() < 0.00001);
            assert!((from_linear(0.0) - 0.0).abs() < 0.00001);
            assert!((from_linear(0.18) - 0.336043).abs() < 0.00001);
            assert!((from_linear(1.0) - 0.513837).abs() < 0.00001);
            assert!((from_linear(10.0) - 0.756599).abs() < 0.00001);
            assert!((from_linear(40.0) - 0.903125).abs() < 0.00001);
            assert!((from_linear(100.0) - 1.0).abs() < 0.00001);
        }

        #[test]
        fn to_linear_test() {
            // Invariants from page 4 of "Wide Gamut Intermediat - DaVinci Resolve 17"
            // from Blackmagic Design, August 2021.
            assert!((to_linear(-0.104443) - -0.01).abs() < 0.00001);
            assert!((to_linear(0.0) - 0.0).abs() < 0.00001);
            assert!((to_linear(0.336043) - 0.18).abs() < 0.00001);
            assert!((to_linear(0.513837) - 1.0).abs() < 0.00001);
            assert!((to_linear(0.756599) - 10.0).abs() < 0.0001);
            assert!((to_linear(0.903125) - 40.0).abs() < 0.001);
            assert!((to_linear(1.0) - 100.0).abs() < 0.001);
        }

        #[test]
        fn round_trip() {
            for i in 0..1024 {
                let n = (i as f32 / 1023.0) * (LINEAR_MAX - LINEAR_MIN) + LINEAR_MIN;
                if n == 0.0 {
                    assert_eq!(to_linear(0.0), 0.0);
                    assert_eq!(from_linear(0.0), 0.0);
                } else {
                    assert!(((n - to_linear(from_linear(n))).abs() / n.abs()) < 0.000_001);
                }
            }
        }
    }
}

/// Blackmagic Design's "4K Film".
pub mod film_4k {
    bmd_log_tf!(
        3.4845696382315063,
        0.035388150275256276,
        0.0797443784368146,
        0.2952978430809614,
        0.781640290185019,
        0.005000044472991669,
        0.03538815,
        -0.010155673,
        2.0150511,
    );
}

/// Blackmagic Design's "4.6K Film Gen 3".
pub mod film_46k_gen3 {
    bmd_log_tf!(
        4.6708570973650385,
        0.07305940817239664,
        0.0287284246696045,
        0.15754052970309015,
        0.6303838233991069,
        0.00499997387034723,
        0.07305941,
        -0.015641542,
        10.416711,
    );
}

/// Blackmagic Design's "Broadcast Film Gen 4".
pub mod broadcast_film_gen4 {
    bmd_log_tf!(
        5.2212906000378565,
        -0.00007134598996420424,
        0.03630411093543444,
        0.21566456116952773,
        0.7133134738229736,
        0.00500072683168086,
        -7.134599e-5,
        1.3664436e-5,
        3.7421572,
    );
}

/// Blackmagic Design's "Film".
pub mod film {
    bmd_log_tf!(
        4.969340550061595,
        0.03538815027497705,
        0.03251848397268609,
        0.1864420102390252,
        0.6723093484094137,
        0.004999977151237935,
        0.03538815,
        -0.007121297,
        5.765991,
    );
}

/// Blackmagic Design's "Pocket 4K Film Gen 4".
pub mod pocket_4k_film_gen4 {
    bmd_log_tf!(
        4.323288448370592,
        0.07305940818036996,
        0.03444835397444396,
        0.1703663112023471,
        0.6454296550413368,
        0.004958295208669562,
        0.07305941,
        -0.016899036,
        7.979818,
    );
}

/// Blackmagic Design's "Pocket 6K Film Gen 4".
pub mod pocket_6k_film_gen4 {
    bmd_log_tf!(
        4.724515510884684,
        0.07305940816299691,
        0.027941380463157067,
        0.15545874964938466,
        0.6272665887366995,
        0.004963316175308281,
        0.07305941,
        -0.015463895,
        10.969201,
    );
}
