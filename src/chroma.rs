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

/// Rec.709/sRGB chromaticities.
pub const REC709: Chromaticities = Chromaticities {
    r: (0.640, 0.330),
    g: (0.300, 0.600),
    b: (0.150, 0.060),
    w: (0.3127, 0.3290),
};

/// Rec.2020 chromaticities.
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

//-------------------------------------------------------------
// Various vendor-specific color spaces.

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

/// Sony's color spaces.
pub mod sony {
    use super::*;
    /// Sony S-Gamut/S-Gamut3 chromaticities.
    ///
    /// Yes, they are exactly the same, as per page 7 of "Technical
    /// Summary for S-Gamut3.Cine/S-Log3 and S-Gamut3/S-Log3" from
    /// Sony.
    pub const S_GAMUT: Chromaticities = Chromaticities {
        r: (0.73000, 0.28000),
        g: (0.14000, 0.85500),
        b: (0.10000, -0.05000),
        w: (0.31270, 0.32900),
    };

    /// Sony S-Gamut3.Cine chromaticities.
    ///
    /// From page 7 of "Technical Summary for S-Gamut3.Cine/S-Log3
    /// and S-Gamut3/S-Log3" from Sony.
    pub const S_GAMUT3_CINE: Chromaticities = Chromaticities {
        r: (0.76600, 0.27500),
        g: (0.22500, 0.80000),
        b: (0.08900, -0.08700),
        w: (0.31270, 0.32900),
    };
}

/// ALEXA Wide Gamut RGB.
///
/// From page 10 of "ALEXA Log C Curve - Usage in VFX" by
/// Harald Brendei, from Arri, 2017-03-09.
pub const ALEXA_WIDE_GAMUT_RGB: Chromaticities = Chromaticities {
    r: (0.6840, 0.3130),
    g: (0.2210, 0.8480),
    b: (0.0861, -0.1020),
    w: (0.3127, 0.3290),
};

/// RED Wide Gamut RGB.
///
/// From page 1 of "White paper on REDWideGamutRGB and Log3G10" from RED.
pub const RED_WIDE_GAMUT_RGB: Chromaticities = Chromaticities {
    r: (0.780308, 0.304253),
    g: (0.121595, 1.493994),
    b: (0.095612, -0.084589),
    w: (0.3127, 0.3290),
};

/// Blackmagic Design's color spaces.
pub mod blackmagic {
    use super::*;

    /// BMD Wide Gamut Gen4/Gen5.
    pub const BMD_WIDE_GAMUT_GEN4: Chromaticities = Chromaticities {
        r: (0.7177, 0.3171),
        g: (0.2280, 0.8616),
        b: (0.1006, -0.0820),
        w: (0.3127, 0.3290),
    };

    /// BMD 4.6k Film Gen3.
    pub const BMD_4_6K_FILM_GEN3: Chromaticities = Chromaticities {
        r: (0.8608, 0.3689),
        g: (0.3282, 0.6156),
        b: (0.0783, -0.0233),
        w: (0.3127, 0.3290),
    };

    /// BMD 4.6k Film Gen1.
    pub const BMD_4_6K_FILM_GEN1: Chromaticities = Chromaticities {
        r: (0.9175, 0.2983),
        g: (0.2982, 1.2835),
        b: (0.0756, -0.0860),
        w: (0.3127, 0.3290),
    };

    /// BMD 4k Film Gen3.
    pub const BMD_4K_FILM_GEN3: Chromaticities = Chromaticities {
        r: (1.0625, 0.3948),
        g: (0.3689, 0.7775),
        b: (0.0956, 0.0332),
        w: (0.3135, 0.3305),
    };

    /// BMD 4k Film Gen1.
    pub const BMD_4K_FILM_GEN1: Chromaticities = Chromaticities {
        r: (0.7422, 0.2859),
        g: (0.4140, 1.3035),
        b: (0.0342, -0.0833),
        w: (0.3135, 0.3305),
    };

    /// BMD Film Gen1.
    pub const BMD_FILM_GEN1: Chromaticities = Chromaticities {
        r: (0.9173, 0.2502),
        g: (0.2833, 1.7072),
        b: (0.0856, -0.0708),
        w: (0.3135, 0.3305),
    };
}
