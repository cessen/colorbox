//! SPI's 1D color transform LUT format.

use std::io::Write;

pub fn write<W: Write>(out: &mut W, range: (f32, f32), table: &[f32]) -> std::io::Result<()> {
    out.write_all(b"Version 1\n")?;
    out.write_all(format!("From {:0.7} {:0.7}\n", range.0, range.1).as_bytes())?;
    out.write_all(format!("Length {}\n", table.len()).as_bytes())?;
    out.write_all(b"Components 1\n")?;
    out.write_all(b"{\n")?;
    for n in table {
        out.write_all(format!("  {:0.7}\n", n).as_bytes())?;
    }
    out.write_all(b"}\n")?;

    Ok(())
}
