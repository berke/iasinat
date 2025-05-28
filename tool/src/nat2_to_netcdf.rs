use super::*;

pub const CMD : Subcommand = Subcommand {
    name:"nat2-to-netcdf",
    synopsis:"Converts a IASI L2 NAT file to a NetCDF file",
    run:run,
    help:"\
    Mandatory arguments:
	--input  IN.nat
		 Path of input IASI L1C file in native (NAT) format
	--output OUT.nc
		 Path of output netCDF file"
};
    
fn run(mut args:Arguments)->Result<()> {
    let input_path : OsString = args.value_from_str("--input")?;
    let output_path : OsString = args.value_from_str("--output")?;

    let rest = args.finish();
    if !rest.is_empty() {
	bail!("Unhandled arguments: {:?}; try --help",rest);
    }

    info!("Opening NAT file {:?}",input_path);
    let fd_in = File::open(&input_path)?;
    let mut br = BufReader::new(fd_in);
    let recs = Grh::read_recs(&mut br)?;
    // let mut iline : usize = 0;

    // Count number of L2 records
    let nline = recs.iter().filter(|rec| {
	matches!(rec.record_kind,GrhRecordKind::MdrL2)
    }).count();
    info!("Number of L2 records: {}",nline);

    // let nchan = ichan1 - ichan0;
    // info!("Number of selected channels: {}",nchan);

    // let dims = (nline,SNOT,PN);

    // macro_rules! mkv {
    // 	($name:ident,$t:ty) => {
    // 	    let mut $name : Array3<$t> = Array3::zeros(dims);
    // 	};
    // 	($name:ident,$t:ty,$x:expr) => {
    // 	    let mut $name : Array3<$t> = Array3::from_elem(dims,$x);
    // 	}
    // }

    // mkv!(lon,f32);
    // mkv!(lat,f32);
    // mkv!(sza,f32);
    // mkv!(saa,f32);
    // mkv!(iza,f32);
    // mkv!(iaa,f32);
    // mkv!(clc,i8);
    // mkv!(lfr,i8);
    // mkv!(sif,i8);

    // let mut rads : Array4<f32> = Array4::zeros((nline,SNOT,PN,nchan));
    // let mut esds : Array1<f64> = Array1::zeros(nline);

    // let mut flg : Array4<i8> = Array4::zeros((nline,SNOT,PN,SB));
    // let mut t0s : Array2<f64> = Array2::zeros((nline,SNOT));
    // let mut wn0_d_wn : Option<(f32,f32)> = None;
    let mut mphr : Option<Mphr> = None;

    // Get GIADR
    let giadr_rec = recs.iter().find(|rec| {
	matches!(rec.record_kind,GrhRecordKind::GiadrL2)
    }).ok_or_else(|| anyhow!("Can't find GIADR"))?;
    let giadr = GiadrL2::read_bin(&mut br,giadr_rec)?;
    println!("{:#?}",giadr);

    for rec in &recs {
	trace!("Record: {:#?}",rec);
	match rec.record_kind {
	    // GrhRecordKind::GiadrL2 => {
	    // 	let giadr_l2 = GiadrL2::read_bin(&mut br,rec)?;
	    // },
	    // GrhRecordKind::GiadrScaleFactors => {
	    // 	sf = Some(GiadrScaleFactors::read_bin(&mut br,rec)?);
	    // },
	    GrhRecordKind::MdrL2 => {
		let mdr_l2 = MdrL2::read_bin(&mut br,&giadr,rec)?;
		println!("{:?}",mdr_l2);
	    },
	    // 	let l1c = MdrL1C::read_bin(&mut br,rec,iline as i32 + 1)?;
	    // 	esds[iline] = l1c.earth_sat_dist as f64;

	    // 	// let spe = SatPosEstimator::new(height)?;
	    // 	if let Some(sf) = sf.as_ref() {
	    // 	    let l1c_rad = MdrL1CRad::read_bin(&mut br,rec,sf)?;

	    // 	    for j in 0..SNOT {
	    // 		let t0 = l1c.cds_date[j].to_unix();

	    // 		t0s[[iline,j]] = t0;
			
	    // 		for i in 0..PN {
	    // 		    let idx = [iline,j,i];
	    // 		    macro_rules! setv {
	    // 			($name:ident) => {
	    // 			    $name[idx] = l1c.$name[j][i];
	    // 			}
	    // 		    }

	    // 		    setv!(lon);
	    // 		    setv!(lat);
	    // 		    setv!(sza);
	    // 		    setv!(saa);
	    // 		    setv!(iza);
	    // 		    setv!(iaa);
	    // 		    setv!(lfr);
	    // 		    setv!(sif);
	    // 		    setv!(clc);

	    // 		    for k in 0..SB {
	    // 			flg[[iline,j,i,k]] = l1c.flg[j][i][k];
	    // 		    }

	    // 		    wn0_d_wn = Some((l1c_rad.wn0,l1c_rad.d_wn));
	    // 		    for k in 0..nchan {
	    // 			rads[[iline,j,i,k]] = l1c_rad.rad[[ichan0 + k,i,j]];
	    // 		    }
	    // 		}
	    // 	    }
	    // 	}

	    // 	iline += 1;
	    // },
	    GrhRecordKind::Mphr => mphr = Some(Mphr::read_bin(&mut br,rec)?),
	    _ => ()
	}
    }

    let mphr = mphr.ok_or_else(|| anyhow!("Could not find MPHR"))?;
    info!("Product name: {}",mphr.product_name);

    info!("Creating NetCDF file {:?}",output_path);
    let mut fd_out = nc::create(&output_path)?;
    
    add_metadata(&mut fd_out,&mphr,"nat2-to-netcdf")?;
    Ok(())
}
