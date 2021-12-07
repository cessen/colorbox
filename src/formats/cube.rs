//! IRIDAS .cube LUT format.
//!
//! This format can contain either a 1D LUT or a 3D LUT, but not both,
//! and there is no indication in the file extension which is which.

use std::io::{BufRead, Write};

pub fn write_1d<W: Write>(
    mut writer: W,
    range_min: [f32; 3],
    range_max: [f32; 3],
    tables: [&[f32]; 3],
) -> std::io::Result<()> {
    assert!(tables[0].len() == tables[1].len() && tables[1].len() == tables[2].len());

    writer.write_all(b"TITLE \"untitled\"\n")?;
    writer.write_all(
        format!(
            "DOMAIN_MIN {:0.7} {:0.7} {:0.7}\n",
            range_min[0], range_min[1], range_min[2],
        )
        .as_bytes(),
    )?;
    writer.write_all(
        format!(
            "DOMAIN_MAX {:0.7} {:0.7} {:0.7}\n",
            range_max[0], range_max[1], range_max[2],
        )
        .as_bytes(),
    )?;
    writer.write_all(format!("LUT_1D_SIZE {}\n", tables[0].len()).as_bytes())?;

    for ((r, g), b) in tables[0]
        .iter()
        .copied()
        .zip(tables[1].iter().copied())
        .zip(tables[2].iter().copied())
    {
        writer.write_all(format!("{:0.7} {:0.7} {:0.7}\n", r, g, b).as_bytes())?;
    }

    Ok(())
}

/// Reads a 1D .cube file.
///
/// Returns (range_min, range_max, table) for each channel.
pub fn read_1d<R: BufRead>(reader: R) -> Result<[(f32, f32, Vec<f32>); 3], super::ReadError> {
    // let mut name: Option<String> = None;
    let mut range_min = [0.0f32; 3];
    let mut range_max = [1.0f32; 3];
    let mut length = None;
    let mut tables = [Vec::new(), Vec::new(), Vec::new()];

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<_> = line.split_whitespace().collect();

        if parts.is_empty() || parts[0].starts_with("#") {
            continue;
        } else if parts[0] == "TITLE" && parts.len() > 1 {
            let name_parts: Vec<_> = line.trim().split("\"").collect();
            if name_parts.len() != 3 || !name_parts[2].is_empty() {
                return Err(super::ReadError::FormatErr);
            }
            // name = Some(name_parts[1].into());
            continue;
        } else if parts[0] == "DOMAIN_MIN" && parts.len() == 4 {
            range_min = [
                parts[1].parse::<f32>()?,
                parts[2].parse::<f32>()?,
                parts[3].parse::<f32>()?,
            ];
            continue;
        } else if parts[0] == "DOMAIN_MAX" && parts.len() == 4 {
            range_max = [
                parts[1].parse::<f32>()?,
                parts[2].parse::<f32>()?,
                parts[3].parse::<f32>()?,
            ];
            continue;
        } else if parts[0] == "LUT_1D_SIZE" && parts.len() == 2 {
            length = Some(parts[1].parse::<usize>()?);
            continue;
        } else if parts.len() == 3 {
            tables[0].push(parts[0].parse::<f32>()?);
            tables[1].push(parts[1].parse::<f32>()?);
            tables[2].push(parts[2].parse::<f32>()?);
            continue;
        } else {
            // Line didn't match any acceptable pattern.
            return Err(super::ReadError::FormatErr);
        }
    }

    let [table_r, table_g, table_b] = tables;
    match length {
        Some(len) if len == table_r.len() => Ok([
            (range_min[0], range_max[0], table_r),
            (range_min[1], range_max[1], table_g),
            (range_min[2], range_max[2], table_b),
        ]),
        _ => Err(super::ReadError::FormatErr),
    }
}
