//! Useful transforms to use as building blocks.

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
pub fn rgb_gamut_intersect(
    from: [f64; 3],
    to: [f64; 3],
    use_ceiling: bool,
    use_floor: bool,
) -> [f64; 3] {
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
