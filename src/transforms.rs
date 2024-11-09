//! Useful transforms to use as building blocks.

/// Converts CIE XYZ coordinates to CIE xyY chromaticity coordinates.
pub fn xyz_to_xyy(xyz: [f64; 3]) -> [f64; 3] {
    let n = xyz[0] + xyz[1] + xyz[2];
    [xyz[0] / n, xyz[1] / n, xyz[1]]
}

/// Converts CIE xyY chromaticity coordinates to CIE XYZ coordinates.
pub fn xyy_to_xyz(xyy: [f64; 3]) -> [f64; 3] {
    let x = xyy[2] / xyy[1] * xyy[0];
    let z = xyy[2] / xyy[1] * (1.0 - xyy[0] - xyy[1]);
    [x, xyy[2], z]
}

/// Operations for working with RGB colors relative to their enclosing gamut.
pub mod rgb_gamut {
    /// Clip an RGB value to an open-domain `[0.0, inf]` color gamut.
    ///
    /// In other words, ensures that all RGB channels are >= 0.0.
    ///
    /// `gray_level` is the achromatic value that we're clipping towards.
    /// For luminance-preserving clipping, this should be the value with
    /// the same luminance as `rgb`.  But client code can more-or-less
    /// compute this however they like for different behaviors.
    ///
    /// `protected` is how much of the gamut is protected from modification.
    /// 0.0 means it's all up for grabs, 1.0 means that the entire gamut is
    /// protected and only out-of-gamut colors are touched.  Lower values make
    /// the mapping softer, larger values sharper.  1.0 is a hard clip.
    pub fn open_domain_clip(rgb: [f64; 3], gray_level: f64, protected: f64) -> [f64; 3] {
        if gray_level <= 0.0 {
            return [0.0; 3];
        }

        // Amount to lerp from `gray_level` -> `rgb` to clip/compress to the gamut boundary.
        let t = {
            let min_component = rgb[0].min(rgb[1]).min(rgb[2]);
            let saturation = (gray_level - min_component) / gray_level;
            if saturation <= 0.0 {
                return rgb;
            }
            let target_saturation = soft_clamp(saturation, protected);

            target_saturation / saturation
        };

        // Do the lerp.
        [
            (gray_level * (1.0 - t)) + (rgb[0] * t),
            (gray_level * (1.0 - t)) + (rgb[1] * t),
            (gray_level * (1.0 - t)) + (rgb[2] * t),
        ]
    }

    /// Clip an RGB value to a closed-domain `[0.0, 1.0]` color gamut.
    ///
    /// Note: this does *not* do open-domain clipping, and assumes that
    /// `rgb` is already within the open-domain gamut (i.e. all channels
    /// are >= 0.0).  If you also need open-domain clipping, do that
    /// *before* passing `rgb` to this function.
    ///
    /// `gray_level` is the achromatic value that we're clipping towards.
    /// For luminance-preserving clipping, this should be the value with
    /// the same luminance as `rgb`.  But client code can more-or-less
    /// compute this however they like for different behaviors.
    ///
    /// `protected` is the channel value up to which all channels are protected
    /// from modification.  1.0 means that the entire closed gamut is protected,
    /// and only values greater than 1.0 are touched, resulting in a hard
    /// clip. Values less than 1.0 give room to smooth out the desaturation
    /// transition, which eliminates mach bands there and generally looks
    /// better, but has to touch some already in-gamut colors to do so (the more
    /// smoothing, the more in-gamut colors are touched).
    pub fn closed_domain_clip(rgb: [f64; 3], gray_level: f64, protected: f64) -> [f64; 3] {
        const EPSILON: f64 = 1.0e-15;

        // Scale the rgb color to be in-gamut, and compute a corresponding gray level.
        let fac = {
            let max_component = rgb[0].max(rgb[1]).max(rgb[2]);
            if max_component <= EPSILON {
                return [0.0; 3];
            }
            soft_clamp(max_component, protected) / max_component
        };
        let scaled_rgb = [rgb[0] * fac, rgb[1] * fac, rgb[2] * fac];
        let scaled_gray_level = gray_level * fac;

        // Mix enough white into the scaled rgb to reach the target gray level.
        let clamped_gray_level = gray_level.clamp(0.0, 1.0);
        if scaled_gray_level >= clamped_gray_level {
            scaled_rgb
        } else {
            let t = ((clamped_gray_level - scaled_gray_level) / (1.0 - scaled_gray_level))
                .clamp(0.0, 1.0);
            [
                (scaled_rgb[0] * (1.0 - t)) + t,
                (scaled_rgb[1] * (1.0 - t)) + t,
                (scaled_rgb[2] * (1.0 - t)) + t,
            ]
        }
    }

    /// Intersects the directed line segment `from` -> `to` with the rgb gamut.
    ///
    /// The intention is to find the closest in-gamut color to `from` on the
    /// line segment.  Thus `to` should typically be an in-gamut color, and
    /// if `from` is already in gamut then `from` is returned.
    ///
    /// - `from`: a possibly out-of-gamut color.
    /// - `to`: a (presumably) in-gamut color.
    /// - `use_ceiling`: if true, the gamut is given a ceiling of rgb
    ///   [1.0, 1.0, 1.0] (bounded luminance).  Otherwise no ceiling.
    /// - `use_floor`: if true, the gamut is given a floor of rgb
    ///   [0.0, 0.0, 0.0] (no negative-luminance colors).  Otherwise
    ///   colors with all negative components are treated as in gamut
    ///   with negative luminance.
    pub fn intersect(from: [f64; 3], to: [f64; 3], use_ceiling: bool, use_floor: bool) -> [f64; 3] {
        // Fast bounding box intersection algorithm often used in ray tracing.
        fn bbox_intersect(
            from: [f64; 3],
            dir_inv: [f64; 3],
            box_min: [f64; 3],
            box_max: [f64; 3],
        ) -> Option<f64> {
            const BBOX_MAXT_ADJUST: f64 = 1.000_000_24;

            // Slab intersections.
            let t1 = [
                (box_min[0] - from[0]) * dir_inv[0],
                (box_min[1] - from[1]) * dir_inv[1],
                (box_min[2] - from[2]) * dir_inv[2],
            ];
            let t2 = [
                (box_max[0] - from[0]) * dir_inv[0],
                (box_max[1] - from[1]) * dir_inv[1],
                (box_max[2] - from[2]) * dir_inv[2],
            ];

            // Near and far hits.
            let far_t = [t1[0].max(t2[0]), t1[1].max(t2[1]), t1[2].max(t2[2])];
            let near_t = [t1[0].min(t2[0]), t1[1].min(t2[1]), t1[2].min(t2[2])];
            let far_hit_t = far_t[0].min(far_t[1]).min(far_t[2]).min(1.0) * BBOX_MAXT_ADJUST;
            let near_hit_t = near_t[0].max(near_t[1]).max(near_t[2]);

            // Check if we hit.
            if near_hit_t <= far_hit_t {
                Some(near_hit_t.max(0.0).min(1.0))
            } else {
                None
            }
        }

        // Compute gamut intersections.
        let dir = [(to[0] - from[0]), (to[1] - from[1]), (to[2] - from[2])];
        let dir_inv = [1.0 / dir[0], 1.0 / dir[1], 1.0 / dir[2]];
        let positive_hit_t = bbox_intersect(
            from,
            dir_inv,
            [0.0; 3],
            if use_ceiling {
                [1.0; 3]
            } else {
                [f64::INFINITY; 3]
            },
        );
        let negative_hit_t = if use_floor {
            None
        } else {
            bbox_intersect(from, dir_inv, [f64::NEG_INFINITY; 3], [0.0; 3])
        };
        let hit_t = match (positive_hit_t, negative_hit_t) {
            (None, None) => {
                return to;
            }
            (Some(t), None) => t,
            (None, Some(t)) => t,
            (Some(t1), Some(t2)) => t1.min(t2),
        };

        // Compute the hit point.
        [
            // Clip to zero for possible floating point rounding error.
            (from[0] + (dir[0] * hit_t)).max(0.0),
            (from[1] + (dir[1] * hit_t)).max(0.0),
            (from[2] + (dir[2] * hit_t)).max(0.0),
        ]
    }

    //---------------------------------------------------------

    /// Clamps `x` to <= 1.0 with a (optionally) smooth transition.
    ///
    /// - `protected`: the value up to which x is not touched.  1.0 is a
    ///    perfectly sharp clip, and lower numbers give room for
    ///    progressively smoother roll-offs.
    ///
    /// https://www.desmos.com/calculator/8nnrcmmxm1
    #[inline(always)]
    fn soft_clamp(x: f64, protected: f64) -> f64 {
        let p = protected; // For brevity.

        if p >= 1.0 || x <= p {
            // `p == 1.0` approaches this, but results in a divide-by-zero
            // when it actually hits it.  So we special-case it.
            // We also do this for `p` or `x` less than zero, because the
            // main equation behaves in unwanted ways in those ranges.
            x.min(1.0)
        } else {
            // Remap.
            let x = (x - p) / (1.0 - p);

            // The main equation.
            let tmp = x / (x * x + 1.0).sqrt();

            // Remap.
            tmp * (1.0 - p) + p
        }
    }
}

/// Open Color IO compatible fixed function transforms.
///
/// The transforms in this module reproduce some of the fixed-function
/// transforms built in to Open Color IO.  They are implemented to
/// exactly reproduce the behavior of the OCIO functions, including all
/// quirks.
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

    /// CIE XYZ -> uvY conversion.
    ///
    /// uvY is the linear Y component from XYZ, and the u' and v' (not u
    /// and v) chromaticity coordinates from CIELUV.
    pub fn xyz_to_uvy(xyz: [f64; 3]) -> [f64; 3] {
        let x = xyz[0];
        let y = xyz[1];
        let z = xyz[2];

        let d = {
            let tmp = x + 15.0 * y + 3.0 * z;
            if tmp == 0.0 {
                0.0
            } else {
                1.0 / tmp
            }
        };

        let u = 4.0 * x * d;
        let v = 9.0 * y * d;

        [u, v, y]
    }

    /// uvY -> CIE XYZ conversion.
    ///
    /// uvY is the linear Y component from XYZ, and the u' and v' (not u
    /// and v) chromaticity coordinates from CIELUV.
    pub fn uvy_to_xyz(uvy: [f64; 3]) -> [f64; 3] {
        let u = uvy[0];
        let v = uvy[1];
        let y = uvy[2];

        let d = if v == 0.0 { 0.0 } else { 1.0 / v };
        let x = (9.0 / 4.0) * y * u * d;
        let z = (3.0 / 4.0) * y * (4.0 - u - (20.0 / 3.0) * v) * d;

        [x, y, z]
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn rgb_hsv_round_trip() {
            for r in -20..21 {
                for g in -20..21 {
                    for b in -20..21 {
                        let rgb = [r as f64 / 1.0, g as f64 / 1.0, b as f64 / 1.0];
                        let hsv = rgb_to_hsv(rgb);
                        let rgb2 = hsv_to_rgb(hsv);
                        if hsv[2] == 0.0 {
                            continue;
                        }
                        assert!((rgb[0] - rgb2[0]).abs() < 1.0e-6);
                    }
                }
            }
        }

        #[test]
        fn xyz_uvy_round_trip() {
            for x in -20..21 {
                for y in -20..21 {
                    for z in -20..21 {
                        let xyz = [x as f64 / 1.0, y as f64 / 1.0, z as f64 / 1.0];
                        let uvy = xyz_to_uvy(xyz);
                        let xyz2 = uvy_to_xyz(uvy);
                        if uvy[1] == 0.0 {
                            continue;
                        }
                        assert!((xyz[0] - xyz2[0]).abs() < 1.0e-6);
                    }
                }
            }
        }
    }
}

/// Transform to/from the OkLab color space.
pub mod oklab {
    use crate::matrix::{transform_color, Matrix};

    /// CIE XYZ -> OkLab.
    ///
    /// Note that OkLab assumes a D65 whitepoint, so input colors with a
    /// different whitepoint should be adapted to that before being
    /// passed.
    #[inline]
    pub fn from_xyz_d65(xyz: [f64; 3]) -> [f64; 3] {
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
        let lms_nonlinear = [
            lms_linear[0].cbrt(),
            lms_linear[1].cbrt(),
            lms_linear[2].cbrt(),
        ];
        transform_color(lms_nonlinear, M2)
    }

    /// OkLab -> CIE XYZ.
    ///
    /// Note that OkLab assumes a D65 whitepoint, so the returned color
    /// will have that whitepoint and should be adapted if desired.
    #[inline]
    pub fn to_xyz_d65(oklab: [f64; 3]) -> [f64; 3] {
        const M1_INV: Matrix = [
            [1.2270138511035211, -0.5577999806518222, 0.2812561489664678],
            [-0.040580178423280586, 1.11225686961683, -0.0716766786656012],
            [-0.0763812845057069, -0.4214819784180127, 1.5861632204407947],
        ];
        const M2_INV: Matrix = [
            [0.9999999984505197, 0.3963377921737678, 0.21580375806075883],
            [1.0000000088817607, -0.10556134232365633, -0.063854174771706],
            [1.000000054672411, -0.08948418209496575, -1.2914855378640917],
        ];

        let lms_nonlinear = transform_color(oklab, M2_INV);
        let lms_linear = [
            lms_nonlinear[0] * lms_nonlinear[0] * lms_nonlinear[0],
            lms_nonlinear[1] * lms_nonlinear[1] * lms_nonlinear[1],
            lms_nonlinear[2] * lms_nonlinear[2] * lms_nonlinear[2],
        ];
        transform_color(lms_linear, M1_INV)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_xyz_d65_01() {
            const TEST_VECS: &[([f64; 3], [f64; 3])] = &[
                ([0.95, 1.0, 1.089], [1.0, 0.0, 0.0]),
                ([1.0, 0.0, 0.0], [0.45, 1.236, -0.019]),
                ([0.0, 1.0, 0.0], [0.922, -0.671, 0.263]),
                ([0.0, 0.0, 1.0], [0.153, -1.415, -0.449]),
            ];
            for (v1, v2) in TEST_VECS.iter().copied() {
                let r1 = from_xyz_d65(v1);
                for i in 0..3 {
                    assert!((r1[i] - v2[i]).abs() < 0.002);
                }
            }
        }

        #[test]
        fn to_xyz_d65_01() {
            const TEST_VECS: &[([f64; 3], [f64; 3])] = &[
                ([0.95, 1.0, 1.089], [1.0, 0.0, 0.0]),
                ([1.0, 0.0, 0.0], [0.45, 1.236, -0.019]),
                ([0.0, 1.0, 0.0], [0.922, -0.671, 0.263]),
                ([0.0, 0.0, 1.0], [0.153, -1.415, -0.449]),
            ];
            for (v1, v2) in TEST_VECS.iter().copied() {
                let r2 = to_xyz_d65(v2);
                for i in 0..3 {
                    assert!((v1[i] - r2[i]).abs() < 0.002);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xyz_xyy_round_trip() {
        let xyz = [0.25, 0.75, 0.5];
        let xyy = xyz_to_xyy(xyz);

        assert_eq!(xyz, xyy_to_xyz(xyy));
    }
}
