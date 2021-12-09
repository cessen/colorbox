/// A 1D look up table.
///
/// The `ranges` specify the input range that the table indices map to.
///
/// The number of `ranges` items should either be 1 or match the number
/// of `tables`.  When there is just one range, it applies to all tables.
/// When there are a matching number, each range corresponds to the
/// respective table.
#[derive(Debug, Clone)]
pub struct Lut1D {
    pub ranges: Vec<(f32, f32)>,
    pub tables: Vec<Vec<f32>>,
}

impl Default for Lut1D {
    fn default() -> Lut1D {
        Lut1D {
            ranges: Vec::new(),
            tables: Vec::new(),
        }
    }
}

/// A 3D lookup table.
///
/// `range` specifies the range of the input cube coordinates on all
/// three axes.  `resolution` specifies the number of samples in each
/// dimension.
///
/// `tables` contains a table for each output component.  Each table has
/// `resolution[0] * resolution[1] * resolution[2]` elements.  The table
/// data is laid out such that the following formula can be used to
/// compute the index of the element at `x,y,z` (or `r,g,b`, etc.):
///
/// ```
/// index = x + (y * resolution[0]) + (z * resolution[0] * resolution[1]);
/// ```
#[derive(Debug, Clone)]
pub struct Lut3D {
    pub range: [(f32, f32); 3],
    pub resolution: [usize; 3],
    pub tables: Vec<Vec<f32>>,
}

impl Default for Lut3D {
    fn default() -> Lut3D {
        Lut3D {
            range: [(0.0, 1.0); 3],
            resolution: [0; 3],
            tables: Vec::new(),
        }
    }
}
