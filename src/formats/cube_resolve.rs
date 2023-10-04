//! Davinci Resolve's .cube LUT format.
//!
//! This format is basically a tweaked version of the original IRIDAS
//! .cube format.
//!
//! It is more flexible than the IRIDAS format in that it can contain
//! both a 3D *and* 1D LUT simultaneously, in addition to containing
//! just one or the other.  But it is less flexible in that the 1D LUTs
//! have a shared rather than individual input range for each channel.

// Implemented according to:
// https://web.archive.org/web/20201125231728/https://forum.blackmagicdesign.com/viewtopic.php?f=21&t=40284

use std::io::{BufRead, Write};

use super::filter_non_finite;
use crate::lut::{Lut1D, Lut3D};

/// Writes a .cube file.
///
/// Can contain either a 1D LUT, a 3D LUT, or both.  But must have at
/// least one of the two.
///
/// - `lut_1d`: (range_min, range_max, tables [r, g, b])
/// - `lut_3d`: (range_min, range_max, resolution, tables [r, g, b]).
///             The tables should have indices ordered the same as the
///             `Lut3D` type.
pub fn write<W: Write>(
    mut writer: W,
    lut_1d: Option<(f32, f32, [&[f32]; 3])>,
    lut_3d: Option<(f32, f32, usize, [&[f32]; 3])>,
) -> std::io::Result<()> {
    assert!(!(lut_1d.is_none() && lut_3d.is_none()));

    // Write header and do basic sanity checks.
    if let Some((range_min, range_max, tables)) = lut_1d {
        assert!(tables[0].len() == tables[1].len() && tables[1].len() == tables[2].len());
        writer.write_all(format!("LUT_1D_SIZE {}\n", tables[0].len()).as_bytes())?;
        writer.write_all(
            format!(
                "LUT_1D_INPUT_RANGE {} {}\n",
                filter_non_finite(range_min),
                filter_non_finite(range_max),
            )
            .as_bytes(),
        )?;
    }
    if let Some((range_min, range_max, res, tables)) = lut_3d {
        assert!(tables[0].len() == (res * res * res));
        assert!(tables[0].len() == tables[1].len() && tables[1].len() == tables[2].len());
        writer.write_all(format!("LUT_3D_SIZE {}\n", res).as_bytes())?;
        writer.write_all(
            format!(
                "LUT_3D_INPUT_RANGE {} {}\n",
                filter_non_finite(range_min),
                filter_non_finite(range_max),
            )
            .as_bytes(),
        )?;
    }

    // Write LUT data.
    if let Some((_, _, tables)) = lut_1d {
        for ((r, g), b) in tables[0]
            .iter()
            .copied()
            .zip(tables[1].iter().copied())
            .zip(tables[2].iter().copied())
        {
            writer.write_all(
                format!(
                    "{} {} {}\n",
                    filter_non_finite(r),
                    filter_non_finite(g),
                    filter_non_finite(b),
                )
                .as_bytes(),
            )?;
        }
    }
    if let Some((_, _, _, tables)) = lut_3d {
        for ((r, g), b) in tables[0]
            .iter()
            .copied()
            .zip(tables[1].iter().copied())
            .zip(tables[2].iter().copied())
        {
            writer.write_all(
                format!(
                    "{} {} {}\n",
                    filter_non_finite(r),
                    filter_non_finite(g),
                    filter_non_finite(b),
                )
                .as_bytes(),
            )?;
        }
    }

    Ok(())
}

/// Reads a .cube file.
///
/// Either a 1D LUT, a 3D LUT, or both can be returned.
pub fn read<R: BufRead>(reader: R) -> Result<(Option<Lut1D>, Option<Lut3D>), super::ReadError> {
    // let mut name: Option<String> = None;
    let mut range_1d = (0.0f32, 1.0f32);
    let mut length_1d = 0;
    let mut tables_1d = [Vec::new(), Vec::new(), Vec::new()];

    let mut range_3d = (0.0f32, 1.0f32);
    let mut size_3d = 0;
    let mut tables_3d = [Vec::new(), Vec::new(), Vec::new()];

    let mut lines = reader.lines().peekable();

    // Parse header.
    while let Some(line) = lines.peek() {
        let line = match line {
            &Ok(ref s) => s,
            &Err(_) => break, // Will be caught later.
        };
        let parts: Vec<_> = line.split_whitespace().collect();

        if parts.is_empty() || parts[0].starts_with("#") {
            // Skip blank lines and comments.
        } else if parts[0] == "TITLE" && parts.len() > 1 {
            let name_parts: Vec<_> = line.trim().split("\"").collect();
            if name_parts.len() != 3 || !name_parts[2].is_empty() {
                return Err(super::ReadError::FormatErr);
            }
            // name = Some(name_parts[1].into());
        } else if parts[0] == "LUT_1D_SIZE" && parts.len() == 2 {
            length_1d = parts[1].parse::<usize>()?;
        } else if parts[0] == "LUT_1D_INPUT_RANGE" && parts.len() == 3 {
            range_1d.0 = parts[1].parse::<f32>()?;
            range_1d.1 = parts[2].parse::<f32>()?;
        } else if parts[0] == "LUT_3D_SIZE" && parts.len() == 2 {
            size_3d = parts[1].parse::<usize>()?;
        } else if parts[0] == "LUT_3D_INPUT_RANGE" && parts.len() == 3 {
            range_3d.0 = parts[1].parse::<f32>()?;
            range_3d.1 = parts[2].parse::<f32>()?;
        } else {
            // Non-header line encountered.  End of header.
            break;
        }

        lines.next();
    }

    // Check for invalid header.
    if length_1d == 0 && size_3d == 0 {
        return Err(super::ReadError::FormatErr);
    }

    let mut length_3d = size_3d * size_3d * size_3d;

    // Parse LUT data.
    for line in lines {
        let line = line?;
        let parts: Vec<_> = line.split_whitespace().collect();

        if parts.is_empty() || parts[0].starts_with("#") {
            // Skip blank lines and comments.
        } else if parts.len() == 3 {
            if length_1d > 0 {
                tables_1d[0].push(parts[0].parse::<f32>()?);
                tables_1d[1].push(parts[1].parse::<f32>()?);
                tables_1d[2].push(parts[2].parse::<f32>()?);
                length_1d -= 1;
            } else if length_3d > 0 {
                tables_3d[0].push(parts[0].parse::<f32>()?);
                tables_3d[1].push(parts[1].parse::<f32>()?);
                tables_3d[2].push(parts[2].parse::<f32>()?);
                length_3d -= 1;
            } else {
                // Should have reached the end already.
                return Err(super::ReadError::FormatErr);
            }
        } else {
            // Line didn't match any acceptable pattern.
            return Err(super::ReadError::FormatErr);
        }
    }

    // Ensure we got the expected amount of data.
    if length_1d > 0 || length_3d > 0 {
        return Err(super::ReadError::FormatErr);
    }

    // Ensure all encountered values were valid (not inf or NaN).
    if !tables_1d.iter().flatten().all(|n| n.is_finite())
        || !tables_3d.iter().flatten().all(|n| n.is_finite())
        || !range_1d.0.is_finite()
        || !range_1d.1.is_finite()
        || !range_3d.0.is_finite()
        || !range_3d.1.is_finite()
    {
        // Non-finite values in the file.
        return Err(super::ReadError::FormatErr);
    }

    // Build the LUT structs.
    let lut_1d = if !tables_1d[0].is_empty() {
        let [table_r, table_g, table_b] = tables_1d;
        Some(Lut1D {
            ranges: vec![range_1d],
            tables: vec![table_r, table_g, table_b],
        })
    } else {
        None
    };
    let lut_3d = if !tables_3d[0].is_empty() {
        let [table_r, table_g, table_b] = tables_3d;
        Some(Lut3D {
            range: [range_3d, range_3d, range_3d],
            resolution: [size_3d, size_3d, size_3d],
            tables: vec![table_r, table_g, table_b],
        })
    } else {
        None
    };

    Ok((lut_1d, lut_3d))
}
