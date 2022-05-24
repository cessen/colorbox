//! Types for storing and working with LUTs in memory.

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

    /// Inverts the LUT, resampling it to the given number of samples.
    ///
    /// This assumes that the table is monotonically increasing.  This
    /// always maintains the same number of `ranges` and `tables` as the
    /// input.
    pub fn resample_inverted(&self, samples: usize) -> Lut1D {
        if self.ranges.len() == 1 {
            let mut lut = Lut1D {
                ranges: vec![(std::f32::INFINITY, -std::f32::INFINITY)],
                tables: Vec::new(),
            };

            // Find our new range.
            for table in self.tables.iter() {
                lut.ranges[0].0 = lut.ranges[0].0.min(table[0]);
                lut.ranges[0].1 = lut.ranges[0].1.max(*table.last().unwrap());
            }

            // Resample the tables.
            for table in self.tables.iter() {
                lut.tables
                    .push(resample_inv(samples, lut.ranges[0], &table, self.ranges[0]));
            }

            lut
        } else if self.ranges.len() == self.tables.len() {
            let mut lut = Lut1D {
                ranges: Vec::new(),
                tables: Vec::new(),
            };

            for (range, table) in self.ranges.iter().zip(self.tables.iter()) {
                let new_range = (table[0], *table.last().unwrap());
                lut.ranges.push(new_range);
                lut.tables
                    .push(resample_inv(samples, new_range, &table, *range));
            }

            lut
        } else {
            panic!("Lut1D range count must either be 1 or match the table count.");
        }
    }

    /// Resample the LUT such that all channels have the same input range.
    ///
    /// The input range of the new LUT will be the union of all the ranges
    /// of the old one.
    pub fn resample_to_single_range(&self, samples: usize) -> Lut1D {
        if self.ranges.len() == 1 && self.tables.iter().all(|t| t.len() == samples) {
            self.clone()
        } else {
            let range = self
                .ranges
                .iter()
                .fold((std::f32::INFINITY, -std::f32::INFINITY), |a, b| {
                    (a.0.min(b.0), a.1.max(b.1))
                });
            let tables: Vec<Vec<f32>> = (0..self.tables.len())
                .map(|i| {
                    resample(
                        samples,
                        range,
                        &self.tables[i],
                        *self.ranges.get(i).unwrap_or(&self.ranges[0]),
                    )
                })
                .collect();

            Lut1D {
                ranges: vec![range],
                tables: tables,
            }
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

/// Helper function for resampling 1D LUTs.
///
/// - `new_samples` is the sample count of the new table.
/// - `new_range_x` is the input range of the new table.
/// - `old_table` is the old table to resample.
/// - `old_range_x` is the input range of the old table.
///
/// New samples outside of the old range will be given the first/last
/// value of the old table.
pub fn resample(
    new_samples: usize,
    new_range_x: (f32, f32),
    old_table: &[f32],
    old_range_x: (f32, f32),
) -> Vec<f32> {
    let mut new_table = Vec::new();

    let offset = (new_range_x.0 - old_range_x.0) / (old_range_x.1 - old_range_x.0);
    let norm = (new_range_x.1 - new_range_x.0) / (old_range_x.1 - old_range_x.0);

    for i in 0..new_samples {
        let x = i as f32 / (new_samples - 1) as f32;

        // Map from new range to old range.  This is the same as:
        // ```
        // let x = ((x * (new_range_x.1 - new_range_x.0)) + new_range_x.0 - old_range_x.0)
        //     / (old_range_x.1 - old_range_x.0);
        // ```
        // Just optimized with precomputed constants.
        let x = offset + (x * norm);

        let y = if x <= 0.0 {
            old_table[0]
        } else if x >= 1.0 {
            *old_table.last().unwrap()
        } else {
            // TODO: conform to the new range.
            let j = x * (old_table.len() - 1) as f32;
            let j1 = j as usize;
            let j2 = j1 + 1;
            if j2 >= old_table.len() {
                // Off the end.
                *old_table.last().unwrap()
            } else {
                // Lerp.
                let alpha = j - j1 as f32;
                (old_table[j1] * (1.0 - alpha)) + (old_table[j2] * alpha)
            }
        };

        new_table.push(y);
    }

    new_table
}

//-------------------------------------------------------------

/// Helper function for inverting 1D LUTs.
///
/// Note that `old_range_x` and `new_range_x` are on different axes,
/// since we're inverting the function.  `new_range_x` corresponds to
/// the y axis of the input table.
fn resample_inv(
    new_samples: usize,
    new_range_x: (f32, f32),
    old_table: &[f32],
    old_range_x: (f32, f32),
) -> Vec<f32> {
    let mut new_table = Vec::new();
    let old_norm = (old_range_x.1 - old_range_x.0) / (old_table.len() - 1) as f32;
    let new_norm = (new_range_x.1 - new_range_x.0) / (new_samples - 1) as f32;

    let mut old_i_1 = 0;
    let mut old_i_2 = 1;
    for i in 0..new_samples {
        let new_x = new_range_x.0 + (i as f32 * new_norm);
        if new_x < old_table[0] {
            new_table.push(old_range_x.0);
        } else if new_x > *old_table.last().unwrap() {
            new_table.push(old_range_x.1);
        } else {
            // Find the interval that contains our new x.
            while new_x > old_table[old_i_2] {
                old_i_1 += 1;
                old_i_2 += 1;
            }

            // Compute the coordinates of the interval ends.
            let old_coords_1 = (
                old_range_x.0 + (old_i_1 as f32 * old_norm),
                old_table[old_i_1],
            );
            let old_coords_2 = (
                old_range_x.0 + (old_i_2 as f32 * old_norm),
                old_table[old_i_2],
            );

            // Interpolate.
            let alpha = {
                let tmp = old_coords_2.1 - old_coords_1.1;
                if tmp > 0.0 {
                    (new_x - old_coords_1.1) / tmp
                } else {
                    0.0
                }
            };
            let new_y = old_coords_1.0 + (alpha * (old_coords_2.0 - old_coords_1.0));
            new_table.push(new_y);
        }
    }

    new_table
}

//-------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn assert_feq(a: f32, b: f32, epsilon: f32) {
        if !((a - b).abs() <= epsilon) {
            panic!("Values not sufficiently equal: a = {}, b = {}", a, b);
        }
    }

    fn make_1d_table<F: Fn(f32) -> f32>(samples: usize, range: (f32, f32), func: F) -> Vec<f32> {
        let norm = ((range.1 as f64 - range.0 as f64) / (samples - 1) as f64) as f32;
        (0..samples)
            .map(|i| func(range.0 + (i as f32 * norm)))
            .collect()
    }

    #[test]
    fn resample_01() {
        let lut1 = vec![0.0, 0.25, 1.0];

        let lut2 = resample(5, (0.0, 1.0), &lut1, (0.0, 1.0));

        assert_eq!(&lut2, &[0.0, 0.125, 0.25, 0.625, 1.0]);
    }

    #[test]
    fn resample_02() {
        let lut1 = vec![0.0, 1.0];

        let lut2 = resample(2, (0.25, 0.75), &lut1, (0.0, 1.0));

        assert_eq!(&lut2, &[0.25, 0.75]);
    }

    #[test]
    fn resample_03() {
        let lut1 = vec![0.0, 1.0];

        let lut2 = resample(3, (-0.25, 1.25), &lut1, (0.0, 1.0));

        assert_eq!(&lut2, &[0.0, 0.5, 1.0]);
    }

    #[test]
    fn resample_04() {
        let lut1 = vec![0.0, 1.0];

        let lut2 = resample(3, (0.25, 1.25), &lut1, (0.25, 1.25));

        assert_eq!(&lut2, &[0.0, 0.5, 1.0]);
    }

    #[test]
    fn resample_05() {
        let lut1 = vec![0.0, 1.0];

        let lut2 = resample(3, (-0.25, 0.75), &lut1, (0.25, 1.25));

        assert_eq!(&lut2, &[0.0, 0.0, 0.5]);
    }

    #[test]
    fn resample_06() {
        let lut1 = vec![0.0, 1.0];

        let lut2 = resample(5, (-0.5, 1.5), &lut1, (0.5, 1.5));

        assert_eq!(&lut2, &[0.0, 0.0, 0.0, 0.5, 1.0]);
    }

    #[test]
    fn resample_07() {
        let lut1 = vec![0.0, 1.0];

        let lut2 = resample(5, (0.5, 1.5), &lut1, (-0.5, 1.5));

        assert_eq!(&lut2, &[0.5, 0.625, 0.75, 0.875, 1.0]);
    }

    #[test]
    fn resample_inv_01() {
        // Ensure resampling to the same effective range works.
        let f = |n: f32| n.log(2.0);
        let f_inv = |n: f32| 2.0_f32.powf(n);

        let samples = 512;
        let range = (0.1, 0.9);
        let lut = make_1d_table(samples, range, f);

        let samples_inv = 113;
        let range_inv = (f(range.0), f(range.1));
        let lut_inv = resample_inv(samples_inv, range_inv, &lut, range);

        let norm_inv = (range_inv.1 - range_inv.0) / (samples_inv - 1) as f32;
        for i in 0..samples_inv {
            let x = range_inv.0 + (i as f32 * norm_inv);
            let y = f_inv(x);
            assert_feq(lut_inv[i], y, 0.00001);
        }
    }

    #[test]
    fn resample_inv_02() {
        // Ensure resampling to a smaller effective range works.
        let f = |n: f32| n.log(2.0);
        let f_inv = |n: f32| 2.0_f32.powf(n);

        let samples = 512;
        let range = (0.1, 0.9);
        let lut = make_1d_table(samples, range, f);

        let samples_inv = 113;
        let range_inv = (f(0.2), f(0.8));
        let lut_inv = resample_inv(samples_inv, range_inv, &lut, range);

        let norm_inv = (range_inv.1 - range_inv.0) / (samples_inv - 1) as f32;
        for i in 0..samples_inv {
            let x = range_inv.0 + (i as f32 * norm_inv);
            let y = f_inv(x);
            assert_feq(lut_inv[i], y, 0.00001);
        }
    }

    #[test]
    fn resample_inv_03() {
        // Ensure resampling to a larger effective range works.
        let f = |n: f32| n.log(2.0);
        let f_inv = |n: f32| 2.0_f32.powf(n);

        let samples = 512;
        let range = (0.1, 0.9);
        let lut = make_1d_table(samples, range, f);

        let samples_inv = 113;
        let range_inv = (f(0.08), f(1.2));
        let lut_inv = resample_inv(samples_inv, range_inv, &lut, range);

        let norm_inv = (range_inv.1 - range_inv.0) / (samples_inv - 1) as f32;
        for i in 20..(samples_inv - 20) {
            let x = range_inv.0 + (i as f32 * norm_inv);
            let y = f_inv(x);
            assert_feq(lut_inv[i], y, 0.00001);
        }

        for i in 0..5 {
            assert_feq(lut_inv[i], range.0, 0.00001);
        }
        for i in (samples_inv - 5)..samples_inv {
            assert_feq(lut_inv[i], range.1, 0.00001);
        }
    }

    #[test]
    fn resample_inv_04() {
        // Ensure interpolation works.
        let f = |n: f32| n * 2.0;
        let f_inv = |n: f32| n / 2.0;

        let samples = 2;
        let range = (-1.0, 2.0);
        let lut = make_1d_table(samples, range, f);

        let samples_inv = 113;
        let range_inv = (f(range.0), f(range.1));
        let lut_inv = resample_inv(samples_inv, range_inv, &lut, range);

        let norm_inv = (range_inv.1 - range_inv.0) / (samples_inv - 1) as f32;
        for i in 0..samples_inv {
            let x = range_inv.0 + (i as f32 * norm_inv);
            let y = f_inv(x);
            assert_feq(lut_inv[i], y, 0.00001);
        }
    }
}
