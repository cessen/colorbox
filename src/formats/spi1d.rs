//! SPI's 1D LUT format.

use std::io::{BufRead, Write};

/// Writes an SPI 1D LUT file.
///
/// Takes a variable number of tables between 1 and 3.  A 1 to
/// 3-component table file will be written depending on the number of
/// tables passed.
pub fn write<W: Write>(
    mut writer: W,
    range_min: f32,
    range_max: f32,
    tables: &[&[f32]],
) -> std::io::Result<()> {
    assert!(tables.len() > 0 && tables.len() <= 3);
    assert!(tables.iter().all(|t| t.len() == tables[0].len()));

    writer.write_all(b"Version 1\n")?;
    writer.write_all(format!("From {:0.7} {:0.7}\n", range_min, range_max).as_bytes())?;
    writer.write_all(format!("Length {}\n", tables[0].len()).as_bytes())?;
    writer.write_all(format!("Components {}\n", tables.len()).as_bytes())?;
    writer.write_all(b"{\n")?;
    for i in 0..tables[0].len() {
        writer.write_all(b" ")?;
        for t in tables.iter() {
            writer.write_all(format!(" {:0.7}", t[i]).as_bytes())?;
        }
        writer.write_all(b"\n")?;
    }
    writer.write_all(b"}\n")?;

    Ok(())
}

/// Reads an SPI 1D LUT file.
///
/// .spi1d supports between 1 and 3 separate component tables.
/// Following the source code in Open Color IO for reading .spi1d files,
/// we always return 3 component tables, which are filled in as follows
/// depending on the number of components in the LUT file:
///
/// - 1 component: all three tables are filled in with the same data
///     from the LUT file.
/// - 2 components: the first two tables are filled in with the
///   corresponding component data, and the third table is filled with
///   zeros.
/// - 3 components: all three tables are filled in with the corresponding
///   component data.
///
/// Returns (range_min, range_max, component_count, tables).
pub fn read<R: BufRead>(reader: R) -> Result<(f32, f32, usize, [Vec<f32>; 3]), super::ReadError> {
    // let mut name: Option<String> = None;
    let mut range_min = 0.0;
    let mut range_max = 1.0;
    let mut length = 0;
    let mut components = 0;
    let mut tables = [Vec::new(), Vec::new(), Vec::new()];
    let mut reading_table = false;

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<_> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        if !reading_table {
            if parts[0] == "Version" && parts.len() == 2 {
                if parts[1].parse::<usize>()? != 1 {
                    return Err(super::ReadError::FormatErr);
                }
                continue;
            } else if parts[0] == "From" && parts.len() == 3 {
                range_min = parts[1].parse::<f32>()?;
                range_max = parts[2].parse::<f32>()?;
                continue;
            } else if parts[0] == "Components" && parts.len() == 2 {
                components = parts[1].parse::<usize>()?;
                continue;
            } else if parts[0] == "Length" && parts.len() == 2 {
                length = parts[1].parse::<usize>()?;
                continue;
            } else if parts[0] == "{" && parts.len() == 1 {
                // Ensure eveything adheres to the format.
                if length == 0 || components < 1 || components > 3 {
                    return Err(super::ReadError::FormatErr);
                }
                reading_table = true;
                continue;
            } else {
                // Line didn't match any acceptable pattern.
                return Err(super::ReadError::FormatErr);
            }
        } else if reading_table {
            if parts[0] == "}" {
                break;
            } else if parts.len() == components {
                match components {
                    1 => {
                        tables[0].push(parts[0].parse::<f32>()?);
                        tables[1].push(parts[0].parse::<f32>()?);
                        tables[2].push(parts[0].parse::<f32>()?);
                    }
                    2 => {
                        tables[0].push(parts[0].parse::<f32>()?);
                        tables[1].push(parts[1].parse::<f32>()?);
                        tables[2].push(0.0);
                    }
                    3 => {
                        tables[0].push(parts[0].parse::<f32>()?);
                        tables[1].push(parts[1].parse::<f32>()?);
                        tables[2].push(parts[2].parse::<f32>()?);
                    }
                    _ => unreachable!(),
                }
                continue;
            } else {
                // Line didn't match any acceptable pattern.
                return Err(super::ReadError::FormatErr);
            }
        }
    }

    if length == tables[0].len() {
        Ok((range_min, range_max, components, tables))
    } else {
        Err(super::ReadError::FormatErr)
    }
}
