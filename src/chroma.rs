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

/// The ACES AP0 chromaticities.
///
/// These are the chromaticities of the ACES2065-1 color space.
pub const ACES_AP0: Chromaticities = Chromaticities {
    r: (0.73470, 0.26530),
    g: (0.00000, 1.00000),
    b: (0.00010, -0.07700),
    w: (0.32168, 0.33767),
};

/// The ACES AP1 chromaticities.
///
/// These are the chromaticities of e.g. the ACEScg, ACEScc, and
/// ACEScct color spaces.
pub const ACES_AP1: Chromaticities = Chromaticities {
    r: (0.713, 0.293),
    g: (0.165, 0.830),
    b: (0.128, 0.044),
    w: (0.32168, 0.33767),
};
