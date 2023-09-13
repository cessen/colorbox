//! Chromaticity coordinates.

pub const WHITEPOINT_E: (f64, f64) = (1.0 / 3.0, 1.0 / 3.0);
pub const WHITEPOINT_D65: (f64, f64) = (0.31271, 0.32902);

/// The chromaticities of a (usually) RGB color space.
///
/// The coordinates are CIE 1931 xy chromaticity coordinates unless
/// otherwise specified.
///
/// `w` is the white point.
#[derive(Debug, Copy, Clone, PartialEq)]
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
    r: (0.71300, 0.29300),
    g: (0.16500, 0.83000),
    b: (0.12800, 0.04400),
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

/// ARRI Wide Gamut 3 (a.k.a. ALEXA Wide Gamut RGB).
pub const ARRI_WIDE_GAMUT_3: Chromaticities = Chromaticities {
    r: (0.6840, 0.3130),
    g: (0.2210, 0.8480),
    b: (0.0861, -0.1020),
    w: (0.3127, 0.3290),
};

/// ARRI Wide Gamut 4.
pub const ARRI_WIDE_GAMUT_4: Chromaticities = Chromaticities {
    r: (0.7347, 0.2653),
    g: (0.1424, 0.8576),
    b: (0.0991, -0.0308),
    w: (0.3127, 0.3290),
};

/// Canon Cinema Gamut.
pub const CANON_CINEMA_GAMUT: Chromaticities = Chromaticities {
    r: (0.7400, 0.2700),
    g: (0.1700, 1.1400),
    b: (0.0800, -0.1000),
    w: (0.3127, 0.3290),
};

/// DJI D-Gamut chromaticities.
pub const DJI_D_GAMUT: Chromaticities = Chromaticities {
    r: (0.7100, 0.3100),
    g: (0.2100, 0.8800),
    b: (0.0900, -0.0800),
    w: (0.3127, 0.3290),
};

/// FilmLight's E-Gamut.
pub const E_GAMUT: Chromaticities = Chromaticities {
    r: (0.8000, 0.3177),
    g: (0.1800, 0.9000),
    b: (0.0650, -0.0805),
    w: (0.3127, 0.3290),
};

/// Panasonic V-Gamut chromaticities.
pub const PANASONIC_V_GAMUT: Chromaticities = Chromaticities {
    r: (0.7300, 0.2800),
    g: (0.1650, 0.8400),
    b: (0.1000, -0.0300),
    w: (0.3127, 0.3290),
};

/// Kodak ProPhoto RGB chromaticities.
pub const PROPHOTO: Chromaticities = Chromaticities {
    r: (0.734699, 0.265301),
    g: (0.159597, 0.840403),
    b: (0.036598, 0.000105),
    w: (0.345704, 0.358540),
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
        r: (0.7177215, 0.3171181),
        g: (0.2280410, 0.8615690),
        b: (0.1005841, -0.0820452),
        w: (0.3127170, 0.3290312),
    };

    /// BMD 4k Film Gen1.
    pub const BMD_4K_FILM_GEN1: Chromaticities = Chromaticities {
        r: (0.7422, 0.2859),
        g: (0.4140, 1.3035),
        b: (0.0342, -0.0833),
        w: (0.3135, 0.3305),
    };

    /// BMD 4k Film Gen3.
    pub const BMD_4K_FILM_GEN3: Chromaticities = Chromaticities {
        r: (1.0625, 0.3948),
        g: (0.3689, 0.7775),
        b: (0.0956, 0.0332),
        w: (0.3135, 0.3305),
    };

    /// BMD 4.6k Film Gen1.
    pub const BMD_4_6K_FILM_GEN1: Chromaticities = Chromaticities {
        r: (0.9175, 0.2983),
        g: (0.2982, 1.2835),
        b: (0.0756, -0.0860),
        w: (0.3127, 0.3290),
    };

    /// BMD 4.6k Film Gen3.
    pub const BMD_4_6K_FILM_GEN3: Chromaticities = Chromaticities {
        r: (0.8608, 0.3689),
        g: (0.3282, 0.6156),
        b: (0.0783, -0.0233),
        w: (0.3127, 0.3290),
    };

    /// BMD Film Gen1.
    pub const BMD_FILM_GEN1: Chromaticities = Chromaticities {
        r: (0.9173, 0.2502),
        g: (0.2833, 1.7072),
        b: (0.0856, -0.0708),
        w: (0.3135, 0.3305),
    };

    /// BMD Pocket 4K Film Gen 4.
    pub const BMD_POCKET_4K_FILM_GEN4: Chromaticities = Chromaticities {
        r: (0.717722, 0.317118),
        g: (0.228041, 0.861569),
        b: (0.100584, -0.082045),
        w: (0.3127, 0.3290),
    };

    /// BMD Video Gen 4.
    pub const BMD_VIDEO_GEN4: Chromaticities = Chromaticities {
        r: (0.682777, 0.318592),
        g: (0.237613, 0.813547),
        b: (0.121743, -0.044283),
        w: (0.3127, 0.3290),
    };

    /// BMD Video Gen 5.
    pub const BMD_VIDEO_GEN5: Chromaticities = Chromaticities {
        r: (0.640000, 0.330000),
        g: (0.300000, 0.600000),
        b: (0.150000, 0.060000),
        w: (0.3127, 0.3290),
    };

    /// DaVinci Wide Gamut BMD Video Gen 5.
    pub const DAVINCI_WIDE_GAMUT: Chromaticities = Chromaticities {
        r: (0.8000, 0.3130),
        g: (0.1682, 0.9877),
        b: (0.0790, -0.1155),
        w: (0.3127, 0.3290),
    };
}

/// Sony's color spaces.
pub mod sony {
    use super::*;
    /// Sony S-Gamut/S-Gamut3 chromaticities.
    ///
    /// Yes, S-Gamut and S-Gamut3 have exactly the same chromaticities,
    /// as per page 7 of "Technical Summary for S-Gamut3.Cine/S-Log3 and
    /// S-Gamut3/S-Log3" from Sony.
    pub const S_GAMUT: Chromaticities = Chromaticities {
        r: (0.7300, 0.2800),
        g: (0.1400, 0.8550),
        b: (0.1000, -0.0500),
        w: (0.3127, 0.3290),
    };

    /// Sony S-Gamut3.Cine chromaticities.
    ///
    /// From page 7 of "Technical Summary for S-Gamut3.Cine/S-Log3
    /// and S-Gamut3/S-Log3" from Sony.
    pub const S_GAMUT3_CINE: Chromaticities = Chromaticities {
        r: (0.7660, 0.2750),
        g: (0.2250, 0.8000),
        b: (0.0890, -0.0870),
        w: (0.3127, 0.3290),
    };
}
