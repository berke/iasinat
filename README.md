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

This tool is in alpha version.  The output has not yet been
extensively checked.

The L1C NetCDF output has been compared to an EUMETSAT NetCDF file and
has been found to be in agreement.  However the EUMETSAT file contents
are partial.

The code converts L1C raw measurements to spectral radiances using the
conversion factors.

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

The tool uses `netcdf`, `simple_logger` and `pico-args`.

The iasinat code contains no unsafe code, nor does it use unchecked
operations.

## Library usage

For using the library in your Rust project, simply do `cargo add iasinat`

## Author

Berke DURAK <bd@exhrd.fr>

## References

- MTG Generic Format Specification, EUM/MTG/SPE/11/0252, v4A
- IASI Level 1 Product Guide, EUM/OPS-EPS/MAN/04/0032, v5
- IASI Level 1 Product Format Specification, EUM.EPS.SYS.SPE.990003, v9E
- IASI Level 2 Product Guide, EUM/OPS-EPS/MAN/04/0033, v3E
- IASI Level 2: Product Format Specification, EPS.MIS.SPE.980760, v9B
