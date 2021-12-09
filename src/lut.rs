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

impl Lut1D {
    /// Creates a single-component 1D LUT from a function and input range.
    pub fn from_fn_1<F: Fn(f32) -> f32>(points: usize, min_x: f32, max_x: f32, f: F) -> Lut1D {
        let inc = (max_x as f64 - min_x as f64) / (points - 1) as f64;
        let mut table = Vec::new();
        for i in 0..points {
            let x = min_x + (inc * i as f64) as f32;
            table.push(f(x));
        }

        Lut1D {
            ranges: vec![(min_x, max_x)],
            tables: vec![table],
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
/// ```ignore
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
