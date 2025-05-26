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
    ellipsoid::{
	WGS84
    }
};

fn main()->Result<()> {
    simple_logger::SimpleLogger::new().init()?;
    let mut args = Arguments::from_env();

    if args.contains("-h") || args.contains("--help") {
	do_help(args)
    } else {
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
         Arguments:
            --input  IN.nat
            --output OUT.nc
";

fn do_help(_args:Arguments)->Result<()> {
    println!("usage: {} SUBCOMMAND ARGS...",
	     std::env::args().next().unwrap());
    println!();
    println!("{}",HELP);
    Ok(())
}

use iasinat_lib::nat::*;
use netcdf as nc;

fn do_nat2nc(mut args:Arguments)->Result<()> {
    let input_path : OsString = args.value_from_str("--input")?;
    let output_path : OsString = args.value_from_str("--output")?;

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

    const NFLG : usize = 3;

    let mut flg : Array4<i8> = Array4::zeros((nline,SNOT,PN,NFLG));
    let mut t0s : Array2<f64> = Array2::zeros((nline,SNOT));

    for rec in &recs {
	trace!("Record: {:#?}",rec);
	match rec.record_kind {
	    GrhRecordKind::GiadrScaleFactors => {
		sf = Some(GiadrScaleFactors::read_bin(&mut br,rec)?);
	    },
	    GrhRecordKind::MdrL1C => {
		iline += 1;
		let l1c = MdrL1C::read_bin(&mut br,rec,iline as i32)?;
		let height = l1c.earth_sat_dist as f64 - WGS84.a;
		trace!("Earth-Satellite distance: {}, height: {}",
		       l1c.earth_sat_dist,
		       height);
		// let spe = SatPosEstimator::new(height)?;
		if let Some(_sf) = sf.as_ref() {
		    // let mut meas_updater = self.meas_loader.make(&mut br,rec,sf)?;
		    for j in 0..SNOT {
			let (jd1,jd2) =
			    (jd2000_1 + l1c.cds_date[j].day as f64,
			     jd2000_2 + l1c.cds_date[j].msec as f64
			     / 86400000.0);
			// let (_gd,fod) = GregorianDate::from_julian(jd1,jd2)?;
			// let _hms = HMS::from_fraction_of_day(fod)?;
			let t0 = ((jd1 - jd_unix_1) +
				  (jd2 - jd_unix_2))*86400.0;

			t0s[[iline - 1,j]] = t0;
			
			for i in 0..PN {
			    // let id = ObservationId {
			    // 	granule:iline as u16,
			    // 	scan:j as u16 + 1,
			    // 	pixel:i as u16 + 1,
			    // };
			    let idx = [iline - 1,j,i];
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
				flg[[iline - 1,j,i,k]] = l1c.flg[j][i][k];
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
			}
		    }
		}
	    },
	    GrhRecordKind::Mphr => {
		let mphr = Mphr::read_bin(&mut br,rec)?;
		trace!("MPHR: {:#?}",mphr);
	    },
	    _ => ()
	}
    }

    info!("Creating NetCDF file {:?}",output_path);
    let mut fd_out = nc::create(&output_path)?;

    fd_out.add_dimension("line",nline)?;
    fd_out.add_dimension("snot",SNOT)?;
    fd_out.add_dimension("pn",PN)?;
    fd_out.add_dimension("flg",NFLG)?;

    macro_rules! putv {
	($x:ident,$t:ty,$units:expr,$long:expr) => {
	    info!("Adding {}",stringify!($x));
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

    info!("Adding flg");
    let mut var = fd_out.add_variable::<i8>("flg",&["line","snot","pn","flg"])?;
    var.put_attribute("long_name","quality flags per band")?;
    var.put(flg.view(),(..,..,..,..))?;

    info!("Adding t0s");
    let mut var = fd_out.add_variable::<f64>("t0s",&["line","snot"])?;
    var.put_attribute("units","second")?;
    var.put_attribute("long_name","unix time of observation")?;
    var.put(t0s.view(),(..,..))?;

    Ok(())
}
