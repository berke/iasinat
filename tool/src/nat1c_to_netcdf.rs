use super::*;

pub const CMD : Subcommand = Subcommand {
    name:"nat1c-to-netcdf",
    synopsis:"Converts a IASI L1C NAT file to a NetCDF file",
    run,
    help:Seq::Cat(&[
	&Seq::One(&"\
Main arguments
==============
--input  IN.nat
	 Path of input IASI L1C file in native (NAT) format
--output OUT.nc - OPTIONAL
	 Path of output netCDF file
         If not given, not netCDF output will be created.
  
Optional arguments
==================

Channel selection
-----------------
By default, all measurement channels will be included in the output.
This can be restricted using the following options:
--ichan0 CH
	 Zero-based index of beginning of IASI channel range
--ichan1 CH
	 Zero-based index of end of IASI channel range (exclusive)

Raw radiances
-------------
Radiances are converted to physical units by applying the scaling
factors contained in the GIADR record and stored as single-precision
floats.

--raw-radiances
	 Create a radiance_raw variable containing the unconverted
	 16-bit signed integer values."),

	#[cfg(feature="footprints")]
	&Seq::Cat(&[
	    &Seq::One(&""),
	    &ell_fp::HELP
	])
    ])
};

pub fn run(mut args:Arguments)->Result<()> {
    let input_path : OsString = args.value_from_str("--input")?;
    let output_path : Option<OsString> = args.opt_value_from_str("--output")?;

    let ichan0 = args.opt_value_from_str("--ichan0")?.unwrap_or(0);
    let ichan1 = args.opt_value_from_str("--ichan1")?.unwrap_or(NBR_IASI);

    let raw_radiances = args.contains("--raw-radiances");

    #[cfg(feature="footprints")]
    let fpp = EllFpProcessor::from_args(&mut args)?;

    finish_args(args)?;

    info!("Opening NAT file {:?}",input_path);
    let mut nat = L1CReader::open(&input_path)?;

    let nline = nat.nline();
    info!("Number of L1C records: {}",nline);

    let nchan = ichan1 - ichan0;
    info!("Number of selected channels: {}",nchan);

    let dims = (nline,SNOT,PN);

    macro_rules! mkv {
	($name:ident,$t:ty) => {
	    let mut $name : Array3<$t> = Array3::zeros(dims);
	};
	($name:ident,$t:ty,$x:expr) => {
	    let mut $name : Array3<$t> = Array3::from_elem(dims,$x);
	}
    }

    mkv!(lon,f32);
    mkv!(lat,f32);
    mkv!(sza,f32);
    mkv!(saa,f32);
    mkv!(iza,f32);
    mkv!(iaa,f32);
    mkv!(clc,i8);
    mkv!(lfr,i8);
    mkv!(sif,i8);

    let mut rads : Array4<f32> = Array4::zeros((nline,SNOT,PN,nchan));
    let mut rads_raw : Array4<i16> =
	if raw_radiances {
	    Array4::zeros((nline,SNOT,PN,nchan))
	} else {
	    Array4::zeros((1,1,1,1))
	};
    let mut esds : Array1<f64> = Array1::zeros(nline);

    let mut flg : Array4<i8> = Array4::zeros((nline,SNOT,PN,SB));
    let mut t0s : Array2<f64> = Array2::zeros((nline,SNOT));
    let mut wn0_d_wn : Option<(f32,f32)> = None;

    for iline in 0..nline {
	let l1c = nat.read_line(iline)?;
	esds[iline] = l1c.earth_sat_dist as f64;

	for j in 0..SNOT {
	    let t0 = l1c.cds_date[j].to_unix();

	    t0s[[iline,j]] = t0;
	    
	    for i in 0..PN {
		let idx = [iline,j,i];
		macro_rules! setv {
		    ($name:ident) => {
			$name[idx] = l1c.$name[j][i];
		    }
		}

		setv!(lon);
		setv!(lat);
		setv!(sza);
		setv!(saa);
		setv!(iza);
		setv!(iaa);
		setv!(lfr);
		setv!(sif);
		setv!(clc);

		for k in 0..SB {
		    flg[[iline,j,i,k]] = l1c.flg[j][i][k];
		}

		wn0_d_wn = Some((l1c.rad.wn0,l1c.rad.d_wn));
		for k in 0..nchan {
		    rads[[iline,j,i,k]] = l1c.rad.rad[[ichan0 + k,i,j]];
		    if raw_radiances {
			rads_raw[[iline,j,i,k]] = l1c.rad.rad_i16[[ichan0 + k,i,j]];
		    }
		}
	    }
	}
    }

    let (wn0,d_wn) = wn0_d_wn.ok_or_else(|| {
	anyhow!("Could not determine wavelength range")
    })?;

    let mphr = nat.mphr();
    info!("Product name: {}",mphr.product_name);
    let t_start = mphr.sensing_start.to_unix();
    let t_end = mphr.sensing_end.to_unix();
    let delta_t = (t_start - t_end)/(nline - 1).max(1) as f64;

    #[cfg(feature="footprints")]
    let fps =
	if fpp.active() {
	    Some(fpp.compute(nline,|iline,j,i| {
		let lon = lon[[iline,j,i]] as f64;
		let lat = lat[[iline,j,i]] as f64;
		let oza = iza[[iline,j,i]] as f64;
		let oaz = iaa[[iline,j,i]] as f64;
		PixelInfo {
		    time_range:(
			t0s[[iline,j]],
			t0s[[iline,j]] + delta_t
		    ),
		    angles:ObservationAngles { lon,lat,oza,oaz },
		    height:esds[iline]
		}
	    })?)
	} else {
	    None
	};

    if let Some(path) = output_path {
	info!("Creating NetCDF file {:?}",path);
	let mut fd_out = nc::create(path)?;
	
	fd_out.add_dimension("line",nline)?;
	fd_out.add_dimension("snot",SNOT)?;
	fd_out.add_dimension("pn",PN)?;
	fd_out.add_dimension("sb",SB)?;
	fd_out.add_dimension("chan",nchan)?;

	macro_rules! putv {
	    ($x:ident,$t:ty,$units:expr,$long:expr) => {
		trace!("Adding {}",stringify!($x));
		let mut var = fd_out.add_variable::<$t>(
		    stringify!($x),&["line","snot","pn"])?;
		var.put_attribute("units",$units)?;
		var.put_attribute("long_name",$long)?;
		var.put($x.view(),(..,..,..))?;
	    }
	}

	putv!(lon,f32,"degree","longitude of pixel center [GGeoSondLoc]");
	putv!(lat,f32,"degree","latitude of pixel center [GGeoSondLoc]");
	putv!(sza,f32,"degree","sun zenith angle [GGeoSondAnglesSUN]");
	putv!(saa,f32,"degree","sun azimuth angle [GGeoSondAnglesSUN]");
	putv!(iza,f32,"degree","observer zenith angle [GGeoSondAnglesMETOP]");
	putv!(iaa,f32,"degree","observer azimuth angle [GGeoSondAnglesMETOP]");
	putv!(clc,i8,"percent","cloud cover [GEUMAvhrr1BCldFrac]");
	putv!(lfr,i8,"percent","land fraction [GEUMAvhrr1BLandFrac]");
	putv!(sif,i8,"bitfield","quality indicator and snow/ice flag [GEUMAvhrr1BQual]");

	trace!("Adding flag");
	let mut var = fd_out.add_variable::<i8>("flag",&["line","snot","pn","sb"])?;
	var.put_attribute("long_name","quality flags per band [GQisFlagQual]")?;
	var.put(flg.view(),(..,..,..,..))?;

	trace!("Adding time");
	let mut var = fd_out.add_variable::<f64>("time",&["line","snot"])?;
	var.put_attribute("units","second")?;
	var.put_attribute("long_name","seconds since Unix epoch [GEPSDatIasi]")?;
	var.put(t0s.view(),(..,..))?;

	trace!("Adding earth_sat_dist");
	let mut var = fd_out.add_variable::<f64>("earth_sat_dist",&["line"])?;
	var.put_attribute("units","meter")?;
	var.put_attribute("long_name","distance from Earth center to satellite \
				       [EARTH_SATELLITE_DISTANCE]")?;
	var.put(esds.view(),..)?;

	trace!("Adding radiances");
	let mut var = fd_out.add_variable::<f32>("radiance",
						 &["line","snot","pn","chan"])?;
	var.put_attribute("units","W/m^2/sr/(cm^-1)")?;
	var.put_attribute("long_name","spectral radiance [GS1cSpect]")?;
	var.put(rads.view(),(..,..,..,..))?;

	if raw_radiances {
	    trace!("Adding raw radiances");
	    let mut var = fd_out.add_variable::<i16>("radiance_raw",
						     &["line","snot","pn","chan"])?;
	    var.put_attribute("units","-")?;
	    var.put_attribute("long_name","raw spectral radiance [GS1cSpect]")?;
	    var.put(rads_raw.view(),(..,..,..,..))?;
	}

	#[cfg(feature="footprints")]
	if let Some(fps) = &fps {
	    fpp.add_to_dataset(&mut fd_out,fps)?;
	}

	trace!("Adding wavenumber");
	let wns = Array1::from_shape_fn(nchan,|k| wn0 + k as f32*d_wn);
	let mut var = fd_out.add_variable::<f32>("wavenumber",&["chan"])?;
	var.put_attribute("units","cm^-1")?;
	var.put_attribute("long_name","channel central wavenumber \
				       [IDefNsFirst,IDefSpectrDWn]")?;
	var.put(wns.view(),..)?;

	add_metadata(&mut fd_out,mphr,"nat1c-to-netcdf")?;
    }

    #[cfg(feature="footprints-mpk")]
    if let Some(fps) = fps {
	fpp.save_mpk(&fps,&mphr)?;
    }
    
    Ok(())
}
