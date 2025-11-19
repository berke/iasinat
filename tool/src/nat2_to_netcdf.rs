use super::*;

pub const CMD : Subcommand = Subcommand {
    name:"nat2-to-netcdf",
    synopsis:"Converts a IASI L2 NAT file to a NetCDF file",
    run,
    help:Seq::Cat(&[
	&Seq::One(&"\
Main arguments
==============
--input      IN.nat
	     Path of input IASI L1C file in native (NAT) format
--output     OUT.nc - OPTIONAL
	     Path of output netCDF file.
             If not given, not netCDF output will be created."),

      #[cfg(feature="footprints")]
      &Seq::One(&"\n\
Optional arguments
==================

"),
      #[cfg(feature="footprints")]
      &ell_fp::HELP
    ])
};
    
fn run(mut args:Arguments)->Result<()> {
    let input_path : OsString = args.value_from_str("--input")?;
    let output_path : Option<OsString> = args.opt_value_from_str("--output")?;

    #[cfg(feature="footprints")]
    let fpp = EllFpProcessor::from_args(&mut args)?;

    finish_args(args)?;

    info!("Opening NAT file {:?}",input_path);
    let mut nat = L2Reader::open(&input_path)?;

    let nline = nat.nline();
    info!("Number of L2 records: {}",nline);

    let giadr = nat.giadr();
    let nlt = giadr.contents.pressure_levels_temp.len();
    let nlq = giadr.contents.pressure_levels_humidity.len();
    let nlo = giadr.contents.pressure_levels_ozone.len();
    let new = giadr.contents.surface_emissivity_wavelengths.len();
    let ncloud = 3;

    let nerrt = giadr.error_data.nerrt();
    let nerrw = giadr.error_data.nerrw();
    let nerro = giadr.error_data.nerro();

    info!("NERRT={}, NERRW={}, NERRO={}",
	  nerrt,
	  nerrw,
	  nerro);

    let mut fg_temp : Array4<f32> = Array4::zeros((nline,SNOT,PN,nlt));
    let mut fg_q : Array4<f64> = Array4::zeros((nline,SNOT,PN,nlq));
    let mut fg_o3 : Array4<f32> = Array4::zeros((nline,SNOT,PN,nlo));
    let mut fg_tsurf : Array3<f32> = Array3::zeros((nline,SNOT,PN));

    let mut temp : Array4<f32> = Array4::zeros((nline,SNOT,PN,nlt));
    let mut q : Array4<f64> = Array4::zeros((nline,SNOT,PN,nlq));
    let mut o3 : Array4<f32> = Array4::zeros((nline,SNOT,PN,nlo));
    let mut tsurf : Array3<f32> = Array3::zeros((nline,SNOT,PN));
    let mut emis : Array4<f32> = Array4::zeros((nline,SNOT,PN,new));
    let mut cc : Array4<f32> = Array4::zeros((nline,SNOT,PN,ncloud));
    let mut ps : Array3<f64> = Array3::zeros((nline,SNOT,PN));
    let mut itconv : Array3<u8> = Array3::zeros((nline,SNOT,PN));
    let mut lansea : Array3<u8> = Array3::zeros((nline,SNOT,PN));
    let mut retcheck : Array3<u16> = Array3::zeros((nline,SNOT,PN));
    let mut surface_z : Array3<f32> = Array3::zeros((nline,SNOT,PN));

    let mut int_q : Array3<f32> = Array3::zeros((nline,SNOT,PN));
    let mut int_o3 : Array3<f32> = Array3::zeros((nline,SNOT,PN));
    let mut int_n2o : Array3<f32> = Array3::zeros((nline,SNOT,PN));
    let mut int_co : Array3<f32> = Array3::zeros((nline,SNOT,PN));
    let mut int_ch4 : Array3<f32> = Array3::zeros((nline,SNOT,PN));
    let mut int_co2 : Array3<f32> = Array3::zeros((nline,SNOT,PN));

    let mut scalt : Array1<f32> = Array1::zeros(nline);

    let mut errt : Array4<f32> = Array4::zeros((nline,SNOT,PN,nerrt));
    let mut errw : Array4<f32> = Array4::zeros((nline,SNOT,PN,nerrw));
    let mut erro : Array4<f32> = Array4::zeros((nline,SNOT,PN,nerro));

    let iang_sza = 0;
    let iang_iza = 1;
    let iang_saz = 2;
    let iang_iaz = 3;
    let nang = 4;

    let ieloc_lat = 0;
    let ieloc_lon = 1;
    let nloc = 2;
    
    let mut ang : Array4<f32> = Array4::zeros((nline,SNOT,PN,nang));
    let mut eloc : Array4<f64> = Array4::zeros((nline,SNOT,PN,nloc));

    let mut nerr_max = 0;
    for iline in 0..nline {
	let mdr_l2 = nat.read_line(iline)?;
	let MdrL2 {
	    first_guess_profiles:MdrL2FirstGuessProfiles {
		fg_atmospheric_temperature:fgat,
		fg_atmospheric_water_vapour:fgaq,
		fg_atmospheric_ozone:fgao,
		fg_surface_temperature:fgts,
		..
	    },
	    measurement_data:MdrL2MeasurementData {
		atmospheric_temperature:at,
		atmospheric_water_vapour:aq,
		atmospheric_ozone:ao,
		surface_temperature:ts,
		integrated_water_vapour:mq,
		integrated_ozone:mo3,
		integrated_n2o:mn2o,
		integrated_co:mco,
		integrated_ch4:mch4,
		integrated_co2:mco2,
		surface_emissivity:memis,
		fractional_cloud_cover:mcc,
		surface_pressure:mps,
		..
	    },
	    navigation_data_scan_line:MdrL2NavigationDataScanLine {
		spacecraft_altitude:mscalt
	    },
	    navigation_data_ifov:MdrL2NavigationDataIfov {
		angular_relation:mang,
		earth_location:meloc
	    },
	    processing_and_quality_flag:MdrL2ProcessingAndQualityFlag {
		flg_itconv:mitconv,
		flg_lansea:mlansea,
		flg_retcheck:mretcheck
	    },
	    error_data:MdrL2ErrorData {
		error_data_index:_,
		temperature_error:merrt,
		water_vapour_error:merrw,
		ozone_error:merro
	    },
            forli_general:MdrL2ForliGeneral {
                surface_z:sz
            },
	    ..
	} = mdr_l2;

	scalt[iline] = mscalt;

	let (_,_,nerr) = merrt.dim();
	nerr_max = nerr_max.max(nerr);
	// trace!("Line {} NERR={}",iline,nerr);
        // trace!("Itconv: {:?}",mitconv);
        // trace!("Lansea: {:?}",mlansea);
        // trace!("Retcheck: {:?}",mretcheck);

	for j in 0..SNOT {
	    for i in 0..PN {
		fg_tsurf[[iline,j,i]] = fgts[[j,i]];
		tsurf[[iline,j,i]] = ts[[j,i]];
		int_q[[iline,j,i]] = mq[[j,i]];
		int_o3[[iline,j,i]] = mo3[[j,i]];
		int_n2o[[iline,j,i]] = mn2o[[j,i]];
		int_co[[iline,j,i]] = mco[[j,i]];
		int_ch4[[iline,j,i]] = mch4[[j,i]];
		int_co2[[iline,j,i]] = mco2[[j,i]];
		ps[[iline,j,i]] = mps[[j,i]];
		itconv[[iline,j,i]] = mitconv[[j,i]];
		lansea[[iline,j,i]] = mlansea[[j,i]];
		retcheck[[iline,j,i]] = mretcheck[[j,i]];
		surface_z[[iline,j,i]] = sz[[j,i]];

		for k in 0..nang {
		    ang[[iline,j,i,k]] = mang[[j,i,k]];
		}

		for k in 0..nloc {
		    eloc[[iline,j,i,k]] = meloc[[j,i,k]];
		}
		
		for k in 0..nlt {
		    fg_temp[[iline,j,i,k]] = fgat[[j,i,k]];
		    temp[[iline,j,i,k]] = at[[j,i,k]];
		}

		for k in 0..nlq {
		    fg_q[[iline,j,i,k]] = fgaq[[j,i,k]];
		    q[[iline,j,i,k]] = aq[[j,i,k]];
		}

		for k in 0..nlo {
		    fg_o3[[iline,j,i,k]] = fgao[[j,i,k]];
		    o3[[iline,j,i,k]] = ao[[j,i,k]];
		}

		for k in 0..new {
		    emis[[iline,j,i,k]] = memis[[j,i,k]];
		}

		for k in 0..ncloud {
		    cc[[iline,j,i,k]] = mcc[[j,i,k]];
		}

                for k in 0..nerrt {
                    errt[[iline,j,i,k]] = merrt[[j,i,k]];
                }

                for k in 0..nerrw {
                    errw[[iline,j,i,k]] = merrw[[j,i,k]];
                }

                for k in 0..nerro {
                    erro[[iline,j,i,k]] = merro[[j,i,k]];
                }
	    }
	}
    }

    let mphr = nat.mphr();
    info!("Product name: {}",mphr.product_name);

    #[cfg(feature="footprints")]
    let fps =
	if fpp.active() {
	    Some(fpp.compute(nline,|iline,j,i| {
		let lon = eloc[[iline,j,i,ieloc_lon]];
		let lat = eloc[[iline,j,i,ieloc_lat]];
		let oza = ang[[iline,j,i,iang_iza]] as f64;
		let oaz = ang[[iline,j,i,iang_iaz]] as f64;
		(ObservationAngles { lon,lat,oza,oaz },
		 scalt[iline] as f64*1e3)
	    })?)
	} else {
	    None
	};

    if let Some(path) = output_path {
	info!("Creating NetCDF file {:?}",path);
	let mut fd_out = nc::create(path)?;

	trace!("Adding dimensions");
	fd_out.add_dimension("line",nline)?;
	fd_out.add_dimension("snot",SNOT)?;
	fd_out.add_dimension("pn",PN)?;

	fd_out.add_dimension("nlt",nlt)?;
	fd_out.add_dimension("nlq",nlq)?;
	fd_out.add_dimension("nlo",nlo)?;
	fd_out.add_dimension("new",new)?;
	fd_out.add_dimension("ncloud",ncloud)?;
	fd_out.add_dimension("nang",nang)?;
	fd_out.add_dimension("nloc",nloc)?;
	fd_out.add_dimension("nerrt",nerrt)?;
	fd_out.add_dimension("nerrw",nerrw)?;
	fd_out.add_dimension("nerro",nerro)?;

	let giadr = nat.giadr();

	let mut var = fd_out.add_variable::<f64>("pressure_levels_temp",&["nlt"])?;
	var.put_values(&giadr.contents.pressure_levels_temp[..],..)?;
	var.put_attribute("long_name","pressure levels on which retrieved \
				       temperature profiles are given")?;
	var.put_attribute("units","Pa")?;

	let mut var = fd_out.add_variable::<f64>("pressure_levels_humidity",
						 &["nlt"])?;
	var.put_values(&giadr.contents.pressure_levels_humidity[..],..)?;
	var.put_attribute("long_name","pressure levels on which retrieved \
				       humidity profiles are given")?;
	var.put_attribute("units","Pa")?;

	let mut var = fd_out.add_variable::<f64>("pressure_levels_ozone",&["nlt"])?;
	var.put_values(&giadr.contents.pressure_levels_ozone[..],..)?;
	var.put_attribute("long_name","pressure levels on which retrieved \
				       ozone profiles are given")?;
	var.put_attribute("units","Pa")?;

	let mut var = fd_out.add_variable::<f64>("surface_emissivity_wavelengths",
						 &["new"])?;
	var.put_values(&giadr.contents.surface_emissivity_wavelengths[..],..)?;
	var.put_attribute("long_name","wavelengths for surface emissivity")?;
	var.put_attribute("units","micrometer")?;

	trace!("Adding FG atmospheric temperature");
	let mut var = fd_out.add_variable::<f32>("fg_atmospheric_temperature",
						 &["line","snot","pn","nlt"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(fg_temp.view(),(..,..,..,..))?;
	var.put_attribute("long_name","a-priori temperature profile")?;
	var.put_attribute("units","K")?;

	trace!("Adding FG atmospheric water vapour");
	let mut var = fd_out.add_variable::<f64>("fg_atmospheric_water_vapour",
						 &["line","snot","pn","nlq"])?;
	var.set_fill_value(f64::NAN)?;
	var.put(fg_q.view(),(..,..,..,..))?;
	var.put_attribute("long_name","a-priori water vapour profile")?;
	var.put_attribute("units","kg/kg")?;

	trace!("Adding FG atmospheric ozone");
	let mut var = fd_out.add_variable::<f32>("fg_atmospheric_ozone",
						 &["line","snot","pn","nlo"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(fg_o3.view(),(..,..,..,..))?;
	var.put_attribute("long_name","a-priori ozone profile")?;
	var.put_attribute("units","kg/kg")?;

	trace!("Adding FG surface temperature");
	let mut var = fd_out.add_variable::<f32>("fg_surface_temperature",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(fg_tsurf.view(),(..,..,..))?;
	var.put_attribute("long_name","a-priori surface skin temperature")?;
	var.put_attribute("units","K")?;
	
	trace!("Adding atmospheric temperature");
	let mut var = fd_out.add_variable::<f32>("atmospheric_temperature",
						 &["line","snot","pn","nlt"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(temp.view(),(..,..,..,..))?;
	var.put_attribute("long_name","temperature profile")?;
	var.put_attribute("units","K")?;

	trace!("Adding atmospheric water vapour");
	let mut var = fd_out.add_variable::<f64>("atmospheric_water_vapour",
						 &["line","snot","pn","nlq"])?;
	var.set_fill_value(f64::NAN)?;
	var.put(q.view(),(..,..,..,..))?;
	var.put_attribute("long_name","water vapour profile")?;
	var.put_attribute("units","kg/kg")?;

	trace!("Adding atmospheric ozone");
	let mut var = fd_out.add_variable::<f32>("atmospheric_ozone",
						 &["line","snot","pn","nlo"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(o3.view(),(..,..,..,..))?;
	var.put_attribute("long_name","ozone profile")?;
	var.put_attribute("units","kg/kg")?;

	trace!("Adding surface temperature");
	let mut var = fd_out.add_variable::<f32>("surface_temperature",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(tsurf.view(),(..,..,..))?;
	var.put_attribute("long_name","surface skin temperature")?;
	var.put_attribute("units","K")?;

	trace!("Adding integrated water vapour");
	let mut var = fd_out.add_variable::<f32>("integrated_water_vapour",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(int_q.view(),(..,..,..))?;
	var.put_attribute("long_name","integrated water vapour")?;
	var.put_attribute("units","kg/m^2")?;

	trace!("Adding integrated ozone");
	let mut var = fd_out.add_variable::<f32>("integrated_ozone",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(int_o3.view(),(..,..,..))?;
	var.put_attribute("long_name","integrated ozone")?;
	var.put_attribute("units","kg/m^2")?;

	trace!("Adding integrated N2O");
	let mut var = fd_out.add_variable::<f32>("integrated_n2o",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(int_n2o.view(),(..,..,..))?;
	var.put_attribute("long_name","integrated N2O")?;
	var.put_attribute("units","kg/m^2")?;

	trace!("Adding integrated CO");
	let mut var = fd_out.add_variable::<f32>("integrated_co",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(int_co.view(),(..,..,..))?;
	var.put_attribute("long_name","integrated CO")?;
	var.put_attribute("units","kg/m^2")?;

	trace!("Adding integrated CH4");
	let mut var = fd_out.add_variable::<f32>("integrated_ch4",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(int_ch4.view(),(..,..,..))?;
	var.put_attribute("long_name","integrated CH4")?;
	var.put_attribute("units","kg/m^2")?;

	trace!("Adding integrated CO2");
	let mut var = fd_out.add_variable::<f32>("integrated_co2",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(int_co2.view(),(..,..,..))?;
	var.put_attribute("long_name","integrated CO2")?;
	var.put_attribute("units","kg/m^2")?;

	trace!("Adding surface emissivity");
	let mut var = fd_out.add_variable::<f32>("surface_emissivity",
						 &["line","snot","pn","new"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(emis.view(),(..,..,..,..))?;
	var.put_attribute("long_name","surface emissivity")?;
	var.put_attribute("units","1")?;

	trace!("Adding fractional cloud cover");
	let mut var = fd_out.add_variable::<f32>("fractional_cloud_cover",
						 &["line","snot","pn","ncloud"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(cc.view(),(..,..,..,..))?;
	var.put_attribute("long_name","fractional cloud cover \
				       (up to 3 cloud formations)")?;
	var.put_attribute("units","%")?;

	trace!("Adding surface pressure");
	let mut var = fd_out.add_variable::<f64>("surface_pressure",
						 &["line","snot","pn"])?;
	var.set_fill_value(f64::NAN)?;
	var.put(ps.view(),(..,..,..))?;
	var.put_attribute("long_name","surface pressure")?;
	var.put_attribute("units","Pa")?;

	trace!("Adding surface elevation");
	let mut var = fd_out.add_variable::<f32>("surface_z",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(surface_z.view(),(..,..,..))?;
	var.put_attribute("long_name","altitude of surface (elevation)")?;
	var.put_attribute("units","m")?;

	trace!("Adding temperature error");
	let mut var = fd_out.add_variable::<f32>("temperature_error",
						 &["line","snot","pn","nerrt"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(errt.view(),(..,..,..,..))?;
	var.put_attribute("long_name","retrieval error covariance matrix for \
				       temperature in principal component domain")?;

	trace!("Adding water vapour error");
	let mut var = fd_out.add_variable::<f32>("water_vapour_error",
						 &["line","snot","pn","nerrw"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(errw.view(),(..,..,..,..))?;
	var.put_attribute("long_name","retrieval error covariance matrix for \
				       water vapour in principal component domain")?;

	trace!("Adding ozone error");
	let mut var = fd_out.add_variable::<f32>("ozone_error",
						 &["line","snot","pn","nerro"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(erro.view(),(..,..,..,..))?;
	var.put_attribute("long_name","retrieval error covariance matrix for \
				       ozone in principal component domain")?;

	trace!("Adding itconv");
	let mut var = fd_out.add_variable::<u8>("flag_itconv",
						&["line","snot","pn"])?;
	var.put(itconv.view(),(..,..,..))?;
	var.put_attribute("long_name","convergence of the iterative retrieval")?;

	trace!("Adding lansea");
	let mut var = fd_out.add_variable::<u8>("flag_lansea",
						&["line","snot","pn"])?;
	var.put(lansea.view(),(..,..,..))?;
	var.put_attribute("long_name","surface type")?;

	trace!("Adding retcheck");
	let mut var = fd_out.add_variable::<u16>("flag_retcheck",
						 &["line","snot","pn"])?;
	var.put(retcheck.view(),(..,..,..))?;
	var.put_attribute("long_name","check that geophysical parameters from the \
				       OEM are within bounds")?;

	trace!("Adding spacecraft altitude");
	let mut var = fd_out.add_variable::<f32>("spacecraft_altitude",
						 &["line"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(scalt.view(),..)?;
	var.put_attribute("long_name","spacecraft altitude above reference geoid \
				       (MSL)")?;
	var.put_attribute("units","km")?;

	trace!("Adding solar_zenith");
	let mut var = fd_out.add_variable::<f32>("solar_zenith",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(ang.slice(s![..,..,..,iang_sza])
		.as_standard_layout()
		.view(),
		(..,..,..))?;
	var.put_attribute("long_name","solar zenith angle")?;
	var.put_attribute("units","degrees")?;

	trace!("Adding satellite_zenith");
	let mut var = fd_out.add_variable::<f32>("satellite_zenith",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(ang.slice(s![..,..,..,iang_iza])
		.as_standard_layout()
		.view(),
		(..,..,..))?;
	var.put_attribute("long_name","satellite zenith angle")?;
	var.put_attribute("units","degrees")?;

	trace!("Adding solar_azimuth");
	let mut var = fd_out.add_variable::<f32>("solar_azimuth",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(ang.slice(s![..,..,..,iang_saz])
		.as_standard_layout()
		.view(),
		(..,..,..))?;
	var.put_attribute("long_name","solar azimuth angle")?;
	var.put_attribute("units","degrees")?;

	trace!("Adding satellite_azimuth");
	let mut var = fd_out.add_variable::<f32>("satellite_azimuth",
						 &["line","snot","pn"])?;
	var.set_fill_value(f32::NAN)?;
	var.put(ang.slice(s![..,..,..,iang_iaz])
		.as_standard_layout()
		.view(),
		(..,..,..))?;
	var.put_attribute("long_name","satellite azimuth angle")?;
	var.put_attribute("units","degrees")?;

	trace!("Adding lat");
	let mut var = fd_out.add_variable::<f64>("lat",
						 &["line","snot","pn"])?;
	var.set_fill_value(f64::NAN)?;
	var.put(eloc.slice(s![..,..,..,ieloc_lat])
		.as_standard_layout()
		.view(),
		(..,..,..))?;
	var.put_attribute("long_name","latitude")?;
	var.put_attribute("units","degrees_north")?;

	trace!("Adding lon");
	let mut var = fd_out.add_variable::<f64>("lon",
						 &["line","snot","pn"])?;
	var.set_fill_value(f64::NAN)?;
	var.put(eloc.slice(s![..,..,..,ieloc_lon])
		.as_standard_layout()
		.view(),
		(..,..,..))?;
	var.put_attribute("long_name","longitude")?;
	var.put_attribute("units","degrees_east")?;

	#[cfg(feature="footprints")]
	if let Some(fps) = &fps {
	    fpp.add_to_dataset(&mut fd_out,fps)?;
	}

	trace!("Adding sensing start and end");
	let _ = fd_out.add_attribute("sensing_start_unix",
				     mphr.sensing_start.to_unix())?;
	let _ = fd_out.add_attribute("sensing_start_timestamp",
				     format!("{}",mphr.sensing_start))?;
	let _ = fd_out.add_attribute("sensing_end_unix",
				     mphr.sensing_end.to_unix())?;
	let _ = fd_out.add_attribute("sensing_end_timestamp",
				     format!("{}",mphr.sensing_end))?;

	add_metadata(&mut fd_out,mphr,"nat2-to-netcdf")?;
    }

    #[cfg(feature="footprints-mpk")]
    if let Some(fps) = fps {
	fpp.save_mpk(&fps)?;
    }
    
    Ok(())
}
