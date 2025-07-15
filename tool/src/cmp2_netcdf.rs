use super::*;

use netcdf_cmp::{
    ComparatorU8,
    ComparatorF64,
    NetcdfCmp
};

pub const CMD : Subcommand = Subcommand {
    name:"cmp2-netcdf",
    synopsis:"Compares EUMETSAT L2 NetCDF files with those produced by this tool",
    run,
    help:"\
    Mandatory arguments:
        --input-eumetsat IN.nc
                 Path of the EUMETSAT L2 file in NetCDF format
        --input-iasinat IN.nc
                 Path of the iasinat L2 NetCDF file"
};

pub fn run(mut args:Arguments)->Result<()> {
    let in1_path : OsString = args.value_from_str("--input-eumetsat")?;
    let in2_path : OsString = args.value_from_str("--input-iasinat")?;
    finish_args(args)?;

    info!("Opening EUMETSAT NetCDF L2 file {:?}",in1_path);
    let fd1 = nc::open(&in1_path)?;
    
    info!("Opening iasinat NetCDF L2 file {:?}",in2_path);
    let fd2 = nc::open(&in2_path)?;

    let product1 : String = fd1.attribute("granule_name")
	.ok_or_else(|| anyhow!("Can't find granule_name attribute"))?
	.value()?
	.try_into()?;
    let product2 : String = fd2.attribute("product_name")
	.ok_or_else(|| anyhow!("Can't find product_name attribute"))?
	.value()?
	.try_into()?;

    if product1 != product2 {
	warn!("Product name mismatch: {} vs {}",product1,product2);
    }

    let cmp = NetcdfCmp::new(fd1,fd2)?;

    macro_rules! cmp1 {
	($name:expr,$tol:expr) => {
	    let cp = ComparatorF64::new($tol);
	    cmp.compare_1d($name,$name,cp)?.check()?;
	}
    }

    macro_rules! cmp23 {
	($name:expr,$tol:expr) => {
	    let cp = ComparatorF64::new($tol);
	    cmp.compare_2d_3d($name,$name,cp)?.check()?;
	}
    }

    macro_rules! cmp23u8 {
	($name:expr,$mask:expr) => {
	    let cp = ComparatorU8::new($mask);
	    cmp.compare_2d_3d($name,$name,cp)?.check()?;
	}
    }

    macro_rules! cmp34 {
	($name:expr,$tol:expr) => {
	    let cp = ComparatorF64::new($tol);
	    cmp.compare_3d_4d($name,$name,cp)?.check()?;
	}
    }

    cmp1!("pressure_levels_temp",1e-6);
    cmp1!("pressure_levels_humidity",1e-6);
    cmp1!("pressure_levels_ozone",1e-6);
    cmp1!("surface_emissivity_wavelengths",1e-6);
    cmp1!("spacecraft_altitude",1e-4);
    cmp34!("fg_atmospheric_temperature",1e-4);
    cmp34!("fg_atmospheric_water_vapour",1e-6);
    cmp34!("fg_atmospheric_ozone",1e-6);
    cmp34!("atmospheric_temperature",1e-4);
    cmp34!("atmospheric_water_vapour",1e-6);
    cmp34!("atmospheric_ozone",1e-6);
    cmp34!("surface_emissivity",1e-6);
    cmp34!("fractional_cloud_cover",1e-4);
    cmp23!("surface_temperature",1e-4);
    cmp23!("surface_pressure",1e-6);
    cmp23!("surface_z",1e-6);
    cmp23!("fg_surface_temperature",1e-4);
    cmp23!("integrated_water_vapour",1e-4);
    cmp23!("integrated_ozone",1e-6);
    cmp23!("integrated_n2o",1e-6);
    cmp23!("integrated_co",1e-6);
    cmp23!("integrated_ch4",1e-6);
    cmp23!("integrated_co2",1e-6);
    cmp23!("lat",1e-9);
    cmp23!("lon",1e-9);
    cmp23!("solar_zenith",1e-4);
    cmp23!("solar_azimuth",1e-4);
    cmp23!("satellite_zenith",1e-4);
    cmp23!("satellite_azimuth",1e-4);
    cmp23u8!("flag_lansea",0xff);

    Ok(())
}
