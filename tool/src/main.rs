use std::{
    fs::File,
    ffi::OsString,
    io::BufReader,
};

use anyhow::{
    bail,
    Result
};

use ndarray::{
    Array2,
    Array3,
    Array4
};

use log::{
    trace,
    info
};

use pico_args::Arguments;

use tofas::{
    calendar::GregorianDate,
};

fn main()->Result<()> {
    let mut args = Arguments::from_env();

    if args.contains("-h") || args.contains("--help") {
	do_help(args)
    } else {
	let verbose = args.contains("--verbose");
	simple_logger::SimpleLogger::new()
	    .with_level(if verbose { log::LevelFilter::Trace }
			else { log::LevelFilter::Info })
	    .init()?;
	match args.subcommand()?.as_deref() {
	    Some("nat2nc") => do_nat2nc(args),
	    Some("help") => do_help(args),
	    Some(cmd) => bail!("Unknown subcommand {}; try --help",cmd),
	    None => bail!("No subcommand specified; try --help")
	}
    }
}

const HELP : &str = r"Subcommands:
nat2nc - Converts a IASI L1C NAT file to a NetCDF file
         Mandatory arguments:
            --input  IN.nat
                     Path of input IASI L1C file in native (NAT) format
            --output OUT.nc
                     Path of output netCDF file

         Optional arguments:
            Channel selection

            By default, all measurement channels will be included in the output.
            This can be restricted using the following options:
            --ichan0 CH
                     Zero-based index of beginning of IASI channel range
            --ichan1 CH
                     Zero-based index of end of IASI channel range (exclusive)
";

fn do_help(_args:Arguments)->Result<()> {
    println!("usage: {} SUBCOMMAND ARGS...",
	     std::env::args().next().unwrap());
    println!();
    println!("{}",HELP);
    Ok(())
}

use iasinat_lib::prelude::*;
use netcdf as nc;

fn do_nat2nc(mut args:Arguments)->Result<()> {
    let input_path : OsString = args.value_from_str("--input")?;
    let output_path : OsString = args.value_from_str("--output")?;

    let ichan0 = args.opt_value_from_str("--ichan0")?.unwrap_or(0);
    let ichan1 = args.opt_value_from_str("--ichan1")?.unwrap_or(NBR_IASI);

    let rest = args.finish();
    if !rest.is_empty() {
	bail!("Unhandled arguments: {:?}; try --help",rest);
    }

    info!("Opening NAT file {:?}",input_path);
    let fd_in = File::open(&input_path)?;
    let mut br = BufReader::new(fd_in);
    let recs = Grh::read_recs(&mut br)?;
    let gd2000 = GregorianDate::new(2000,1,1)?;
    let gd_unix = GregorianDate::new(1970,1,1)?;
    let (jd2000_1,jd2000_2) = gd2000.to_julian();
    let (jd_unix_1,jd_unix_2) = gd_unix.to_julian();
    let mut iline : usize = 0;
    let mut sf = None;

    // Count number of L1C records
    let nline = recs.iter().filter(|rec| {
	matches!(rec.record_kind,GrhRecordKind::MdrL1C)
    }).count();
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
    mkv!(lfr,i8);
    mkv!(sif,i8);

    let mut rads : Array4<f32> = Array4::zeros((nline,SNOT,PN,nchan));
    let mut esds : Array1<f64> = Array1::zeros(nline);

    const NFLG : usize = 3;

    let mut flg : Array4<i8> = Array4::zeros((nline,SNOT,PN,NFLG));
    let mut t0s : Array2<f64> = Array2::zeros((nline,SNOT));
    let mut wn0_d_wn : Option<(f32,f32)> = None;
    let mut mphr : Option<Mphr> = None;

    for rec in &recs {
	trace!("Record: {:#?}",rec);
	match rec.record_kind {
	    GrhRecordKind::GiadrScaleFactors => {
		sf = Some(GiadrScaleFactors::read_bin(&mut br,rec)?);
	    },
	    GrhRecordKind::MdrL1C => {
		let l1c = MdrL1C::read_bin(&mut br,rec,iline as i32 + 1)?;
		esds[iline] = l1c.earth_sat_dist as f64;

		// let spe = SatPosEstimator::new(height)?;
		if let Some(sf) = sf.as_ref() {
		    let l1c_rad = MdrL1CRad::read_bin(&mut br,rec,sf)?;

		    for j in 0..SNOT {
			let (jd1,jd2) =
			    (jd2000_1 + l1c.cds_date[j].day as f64,
			     jd2000_2 + l1c.cds_date[j].msec as f64
			     / 86400000.0);
			// let (_gd,fod) = GregorianDate::from_julian(jd1,jd2)?;
			// let _hms = HMS::from_fraction_of_day(fod)?;
			let t0 = ((jd1 - jd_unix_1) +
				  (jd2 - jd_unix_2))*86400.0;

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

			    for k in 0..NFLG {
				flg[[iline,j,i,k]] = l1c.flg[j][i][k];
			    }

			    // let est = spe.estimate(lon as f64,
			    // 			   lat as f64,
			    // 			   oza as f64,
			    // 			   oaz as f64)?;

			    // let el = Ellipse::from_estimate(&self.ec,
			    // 				    self.hca_ifov as f64,
			    // 				    &est)?;

			    // let footprint =
			    // 	if self.rough_footprints {
			    // 	    let footprint = EllipticalFootprint::from_estimate(
			    // 		self.hca_ifov as f64,
			    // 		&est);
			    // 	    ste_dcenter.add(el.center.norm2());
			    // 	    ste_da.add(abs(el.a - footprint.a));
			    // 	    ste_db.add(abs(el.b - footprint.b));
			    // 	    ste_dpa.add(
			    // 		abs((el.theta - footprint.pa + 90.0)
			    // 		    .rem_euclid(180.0) - 90.0));
			    // 	    if false {
			    // 		println!("center = {}",el.center);
			    // 		println!("a      = {:14.6e}",el.a);
			    // 		println!("a'     = {:14.6e}",footprint.a);
			    // 		println!("b      = {:14.6e}",el.b);
			    // 		println!("b'     = {:14.6e}",footprint.b);
			    // 		println!("theta  = {:+6.3}",el.theta);
			    // 		println!("theta' = {:+6.3}",footprint.pa);
			    // 	    }
			    // 	    footprint
			    // 	} else {
			    // 	    EllipticalFootprint {
			    // 		a:el.a,
			    // 		b:el.b,
			    // 		pa:el.theta
			    // 	    }
			    // 	};

			    // let aux = IasiAux {
			    // 	flg,
			    // 	lfr,
			    // 	sif,
			    // 	p_sat:est.p_sat
			    // };
			    // let obs = Observation {
			    // 	lat,
			    // 	lon,
			    // 	t0,
			    // 	oza,
			    // 	sza,
			    // 	saz,
			    // 	oaz,
			    // 	cf,
			    // 	alt:None,
			    // 	footprint,
			    // 	meas:IasiMeas::Aux(aux)
			    // };
			    // if filter(&id,&obs) {
			    // 	let obs = meas_updater
			    // 	    .update(&mut br,rec,sf,&id,obs)?;
			    // 	product.insert(id,obs);
			    // }

			    wn0_d_wn = Some((l1c_rad.wn0,l1c_rad.d_wn));
			    for k in 0..nchan {
				rads[[iline,j,i,k]] = l1c_rad.rad[[ichan0 + k,i,j]];
			    }
			}
		    }
		}

		iline += 1;
	    },
	    GrhRecordKind::Mphr => mphr = Some(Mphr::read_bin(&mut br,rec)?),
	    _ => ()
	}
    }

    let (wn0,d_wn) = wn0_d_wn.ok_or_else(|| {
	anyhow!("Could not determine wavelength range")
    })?;

    let mphr = mphr.ok_or_else(|| anyhow!("Could not find MPHR"))?;
    info!("Product name: {}",mphr.product_name);

    info!("Creating NetCDF file {:?}",output_path);
    let mut fd_out = nc::create(&output_path)?;
    
    fd_out.add_dimension("line",nline)?;
    fd_out.add_dimension("snot",SNOT)?;
    fd_out.add_dimension("pn",PN)?;
    fd_out.add_dimension("flg",NFLG)?;
    fd_out.add_dimension("chan",nchan)?;

    macro_rules! putv {
	($x:ident,$t:ty,$units:expr,$long:expr) => {
	    trace!("Adding {}",stringify!($x));
	    let mut var = fd_out.add_variable::<$t>(stringify!($x),&["line","snot","pn"])?;
	    var.put_attribute("units",$units)?;
	    var.put_attribute("long_name",$long)?;
	    var.put($x.view(),(..,..,..))?;
	}
    }

    putv!(lon,f32,"degree","longitude of pixel center");
    putv!(lat,f32,"degree","latitude of pixel center");
    putv!(sza,f32,"degree","sun zenith angle");
    putv!(saa,f32,"degree","sun azimuth angle");
    putv!(iza,f32,"degree","observer zenith angle");
    putv!(iaa,f32,"degree","observer azimuth angle");
    putv!(lfr,i8,"percent","land fraction");
    putv!(sif,i8,"?","fluorescence");

    trace!("Adding flag");
    let mut var = fd_out.add_variable::<i8>("flag",&["line","snot","pn","flg"])?;
    var.put_attribute("long_name","quality flags per band")?;
    var.put(flg.view(),(..,..,..,..))?;

    trace!("Adding time");
    let mut var = fd_out.add_variable::<f64>("time",&["line","snot"])?;
    var.put_attribute("units","second")?;
    var.put_attribute("long_name","seconds since Unix epoch")?;
    var.put(t0s.view(),(..,..))?;

    trace!("Adding earth_sat_dist");
    let mut var = fd_out.add_variable::<f64>("earth_sat_dist",&["line"])?;
    var.put_attribute("units","meter")?;
    var.put_attribute("long_name","distance from Earth center to satellite")?;
    var.put(esds.view(),..)?;

    trace!("Adding radiance");
    let mut var = fd_out.add_variable::<f32>("radiance",&["line","snot","pn","chan"])?;
    var.put_attribute("units","W/m^2/sr/(cm^-1)")?;
    var.put_attribute("long_name","spectral radiance")?;
    var.put(rads.view(),(..,..,..,..))?;

    trace!("Adding wavenumber");
    let wns = Array1::from_shape_fn(nchan,|k| wn0 + k as f32*d_wn);
    let mut var = fd_out.add_variable::<f32>("wavenumber",&["chan"])?;
    var.put_attribute("units","cm^-1")?;
    var.put_attribute("long_name","channel central wavenumber")?;
    var.put(wns.view(),..)?;

    trace!("Adding metadata");

    let name = "iasinat by ExH R&D S.A.R.L. <bd@exhrd.fr>";

    let _attr = fd_out.add_attribute("product_name",mphr.product_name.clone())?;
    let _conv_name = fd_out.add_attribute("converter_name",name.to_string());
    let _conv_commit = fd_out.add_attribute(
	"converter_commit",
	env!("IASINAT_COMMIT").to_string());
    let _conv_stamp = fd_out.add_attribute(
	"converter_build_timestamp",
	env!("IASINAT_BUILD_TIMESTAMP").to_string());
    Ok(())
}
