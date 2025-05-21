# Changelog


## [Unreleased]

- Different parameterization for gamut clipping softness, that gives hard guarantees on how much of the gamut is left untouched.
- Added chromaticity coordinates for more standard illuminants.
- Renamed the matrix function `invert()` to `inverse()`.


## [0.4.0] - 2024-02-21

### Additions

- Misc convenience functions for building and working with LUTs.
- A bunch more chromaticities and transfer functions, including those from ARRI.
- A new "transforms" module with various useful color transforms.

### Improvements

- The Resolve .cube reader is now more lenient in what it accepts.

### Other Changes

- `matrix_compose` is now a function instead of a macro, and simply takes a slice of matrices.


## [0.3.0] - 2022-05-26

### Additions

- Transfer functions: Panasonic V-Log, Fujifilm F-Log, DJI D-Log, Nikon N-Log.
- Color space chromatcity coordinates: Blackmagic Design's various spaces, Panasonic V-Gamut, DJI D-Gamut.
- Tables of the Stockman & Sharpe 10-degree cone fundamentals.
- Misc. helper funtions for working with 1D LUTs.
- Support for Davinci Resolve's custom .cube format variant.
- Support for the 3D variant of the original Iridas .cube format.


## [0.2.0] - 2021-12-13

### Additions

- Chromaticities of more color spaces.
- Functions to read and write some common LUT formats.
- Types for working with LUTs in memory.
- Functions to convert matrices to more common data layouts.
- A macro to compose multiple matrices together.
- Tables of the CIE 1931 XYZ spectral sensitivity curves.


## [0.1.0] - 2021-12-04

### First Release

- Functions for building color space transform matrices, for converting between color spaces.
- Functions for building chromatic adaptation matrices.
- Functions to write a couple of different 1D LUT formats.
- A collection of common color space chromaticities.
- A collection of common transfer functions.


[Unreleased]: https://github.com/cessen/colorbox/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/cessen/colorbox/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/cessen/colorbox/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/cessen/colorbox/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/cessen/colorbox/releases/tag/v0.1.0
