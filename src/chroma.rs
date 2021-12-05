//! Chromaticity coordinates.

/// The chromaticities of a (usually) RGB color space.
///
/// The coordinates are CIE 1931 xy chromaticity coordinates unless
/// otherwise specified.
///
/// `w` is the white point.
#[derive(Debug, Copy, Clone)]
pub struct Chromaticities {
    pub r: (f64, f64),
    pub g: (f64, f64),
    pub b: (f64, f64),
    pub w: (f64, f64),
}

/// Chromaticities of the Rec.709 and sRGB color spaces.
pub const REC709: Chromaticities = Chromaticities {
    r: (0.640, 0.330),
    g: (0.300, 0.600),
    b: (0.150, 0.060),
    w: (0.3127, 0.3290),
};

/// Chromaticities of the Rec.2020 color space.
pub const REC2020: Chromaticities = Chromaticities {
    r: (0.708, 0.292),
    g: (0.170, 0.797),
    b: (0.131, 0.046),
    w: (0.3127, 0.3290),
};

/// DCI-P3 chromaticities.
pub const DCI_P3: Chromaticities = Chromaticities {
    r: (0.680, 0.320),
    g: (0.265, 0.690),
    b: (0.150, 0.060),
    w: (0.314, 0.351),
};

/// ACES AP0 chromaticities.
///
/// These are the chromaticities of the ACES2065-1 color space.
pub const ACES_AP0: Chromaticities = Chromaticities {
    r: (0.73470, 0.26530),
    g: (0.00000, 1.00000),
    b: (0.00010, -0.07700),
    w: (0.32168, 0.33767),
};

/// ACES AP1 chromaticities.
///
/// These are the chromaticities of e.g. the ACEScg, ACEScc, and
/// ACEScct color spaces.
pub const ACES_AP1: Chromaticities = Chromaticities {
    r: (0.713, 0.293),
    g: (0.165, 0.830),
    b: (0.128, 0.044),
    w: (0.32168, 0.33767),
};

/// Adobe RGB chromaticities.
pub const ADOBE_RGB: Chromaticities = Chromaticities {
    r: (0.6400, 0.3300),
    g: (0.2100, 0.7100),
    b: (0.1500, 0.0600),
    w: (0.3127, 0.3290),
};

/// Adobe Wide-gamut RGB chromaticities.
pub const ADOBE_WIDE_GAMUT_RGB: Chromaticities = Chromaticities {
    r: (0.7347, 0.2653),
    g: (0.1152, 0.8264),
    b: (0.1566, 0.0177),
    w: (0.3457, 0.3585),
};

/// Kodak ProPhoto RGB chromaticities.
pub const PROPHOTO: Chromaticities = Chromaticities {
    r: (0.734699, 0.265301),
    g: (0.159597, 0.840403),
    b: (0.036598, 0.000105),
    w: (0.345704, 0.358540),
};
