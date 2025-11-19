# iasinat

This is a library for reading IASI level 1C and level 2 files in
EUMETSAT's so-called "NAT" binary file format.

There is also a tool "iasinat" for converting L1C and L2 files to
NetCDF.

This code was developed in collaboration with SPASCIA under the
CNES CH4 SWIR-TIR contract.

## Status and completeness

Currently this code only handles a subset of the available data, based
on the needs of the SWIR-TIR and related projects.

The library can read MPHR, GIADR and MDR records.

This tool is in a beta state, except for footprint generation, which
is alpha.

The outputs of this tool have been compared on a number of NAT files
to NetCDF files provided by EUMETSAT or converted using CODA.  SPASCIA
did some preliminary sanity-checking of the posterior errors.

### Level 1C

The L1C NetCDF output from this tool has been found to be in agreement
with NetCDF files provided by EUMETSAT, but the files are partial, as
some of the information present in the NAT files is not present in the
NetCDF files.

The iasinat tool converts L1C raw measurements to spectral radiances
using the conversion factors.

### Level 2

As for level 1C, the level 2 output is partial (but fairly complete) and
is in complete agreement with the EUMETSAT Level2 output, as well as
CODA output, except for the error data.

The current EUMETSAT L2 product format specification and product guide
documents incorrectly describe the error data records.  Contrary to
what is stated in these documents, the data records for temperature,
water vapour and ozone do not have fixed sizes.  The CODA description
handles these variable-length records, but fails to reorder the
vectors so that they can be attributed to individual pixels.

This tool handles both issues, but due to a lack of information we
have not yet been able to fully validate its output.

## Build instructions for the tool

- The code is written in Rust, you need to install the Rust package
  manager `cargo`
- For the tool, you need libnetcdf (`apt install libnetcdf-dev`
  on Debian-like systems)
- Checkout the repository and simply run `cargo build --release`

## Dependencies

The dependencies of the library are minimal:

- `anyhow` for error handling
- `log` for logging (error messages etc.)
- `ndarray` for handling multi-dimensional arrays
- `regex` for parsing product header strings
- `tofas` for Julian date/time conversions
- `circfp` (optional) for computing elliptical footprints (see below)
- `footprint` (optional) for exporting footprints in an MPK format
  (see below)

The tool uses `netcdf`, `simple_logger` and `pico-args`.

The iasinat code contains no unsafe code, nor does it use unchecked
operations.

Starting from version 0.1.2, the tool can be compiled with the
"footprints" feature, which uses `circfp` to compute the elliptical
footprints.

From version 0.1.3, the tool can be compiled with the "footprints-mpk"
feature, which enables the footprints to be exported in the MPK format
used by the `footprint` crate.

## Library usage

For using the library in your Rust project, simply do `cargo add iasinat`

## Version history

| Version | Date       | Comment                    |
|---------|------------|----------------------------|
| 0.1.0   | 2025-06-30 | Initial release            |
| 0.1.1   | 2025-07-15 | Add surface_z              |
| 0.1.2   | 2025-11-13 | Add footprints feature     |
| 0.1.3   | 2024-11-19 | Add footprints-mpk feature |
| 0.1.4   | 2024-11-19 | Fix height error           |

## Author

Berke DURAK <bd@exhrd.fr>

## References

- MTG Generic Format Specification, EUM/MTG/SPE/11/0252, v4A
- IASI Level 1 Product Guide, EUM/OPS-EPS/MAN/04/0032, v5
- IASI Level 1 Product Format Specification, EUM.EPS.SYS.SPE.990003, v9E
- IASI Level 2 Product Guide, EUM/OPS-EPS/MAN/04/0033, v3E
- IASI Level 2: Product Format Specification, EPS.MIS.SPE.980760, v9B
