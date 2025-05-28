use super::*;

pub const CMD : Subcommand = Subcommand {
    name:"cmp1c-netcdf",
    synopsis:"Compares EUMETSAT L1C NetCDF files with those produced by this tool",
    run,
    help:"\
    Mandatory arguments:
        --input-eumetsat IN.nc
                 Path of the EUMETSAT L1C file in NetCDF format
        --input-iasinat IN.nc
                 Path of the iasinat NetCDF file"
};

pub fn run(mut args:Arguments)->Result<()> {
    let in1_path : OsString = args.value_from_str("--input-eumetsat")?;
    let in2_path : OsString = args.value_from_str("--input-iasinat")?;
    finish_args(args)?;

    info!("Opening EUMETSAT NetCDF file {:?}",in1_path);
    let fd1 = nc::open(&in1_path)?;
    
    info!("Opening iasinat NetCDF file {:?}",in2_path);
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

    let cmp_vars = |name1:&str,name2:&str,tol:f32|->Result<()> {
	let val1_var = fd1.variable(name1)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 1",name1))?;
	let val1 = val1_var.get::<f32,_>((..,..))?;

	let val2_var = fd2.variable(name2)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 2",name2))?;
	let val2 = val2_var.get::<f32,_>((..,..,..))?;
	let mut ntot = 0;
	let mut nbad = 0;
	let mut e_max : f32 = 0.0;
	const NBAD_MAX : usize = 10;
	if let &[nline,snot,pn] = val2.dim().slice() {
	    if let &[_nalong,_nacross] = val1.dim().slice() {
		let val1 : Array3<f32> = val1.into_shape_with_order((nline,snot,pn))?;
		let val2 : Array3<f32> = val2.into_shape_with_order((nline,snot,pn))?;

		for iline in 0..nline {
		    for j in 0..snot {
			for i in 0..pn {
			    ntot += 1;
			    let l1 = val1[[iline,j,i]];
			    let l2 = val2[[iline,j,i]];
			    let e = (l1 - l2).abs();
			    e_max = e_max.max(e);
			    if e > tol && nbad < NBAD_MAX {
				warn!("At {},{},{}: {} = {:.6} vs {} = {:.6} \
				       (e={:.3e})",
				      iline,j,i,
				      name1,
				      l1,
				      name2,
				      l2,
				      e);
				nbad += 1;
			    }
			}
		    }
		}
	    } else {
		bail!("Can't reshape {}",name1);
	    }
	} else {
	    bail!("Can't reshape {}",name2);
	}
	info!("Variable {} vs {}: total {}, bad {}, e_max {:.3e}",
	      name1,
	      name2,
	      ntot,
	      nbad,
	      e_max);
	Ok(())
    };

    cmp_vars("lat","lat",1e-4)?;
    cmp_vars("lon","lon",1e-4)?;
    cmp_vars("pixel_zenith_angle","iza",1e-4)?;
    cmp_vars("pixel_azimuth_angle","iaa",1e-4)?;
    cmp_vars("pixel_solar_zenith_angle","sza",1e-4)?;
    cmp_vars("pixel_solar_azimuth_angle","saa",1e-4)?;
    Ok(())
}
