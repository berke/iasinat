use super::*;

#[derive(Debug)]
pub struct MdrL1C {
    // instrument mode
    pub geps_iasi_mode:i32,
    // scan position
    pub geps_sp:[i32;SNOT],
    // Corner cube direction
    pub geps_ccd:[i8;SNOT],
    // longitude, latitude
    pub lon:[[f32;PN];SNOT],
    // longitude, latitude
    pub lat:[[f32;PN];SNOT],
    // days since 2001-01-01
    pub cds_date:[ShortCdsTime;SNOT],
    // quality flag per band
    pub flg:[[[i8;SB];PN];SNOT],
    // instrument azimuth and zenith angles
    pub iaa:[[f32;PN];SNOT],
    // instrument azimuth and zenith angles
    pub iza:[[f32;PN];SNOT],
    // solar azimuth and zenith angles
    pub saa:[[f32;PN];SNOT],
    // solar azimuth and zenith angles
    pub sza:[[f32;PN];SNOT],
    // cloud cover, land fraction, AVHRR 1B qual
    pub clc:[[i8;PN];SNOT],
    // cloud cover, land fraction, AVHRR 1B qual
    pub lfr:[[i8;PN];SNOT],
    // cloud cover, land fraction, AVHRR 1B qual
    pub sif:[[i8;PN];SNOT],
    // Earth-Satellite distance [m]
    pub earth_sat_dist:u32
}

#[derive(Debug)]
pub struct MdrL1CRad {
    // wavenumber of first sample
    pub wn0:f32,
    // spectral step
    pub d_wn:f32,
    // first sample
    pub ns_first:i32,
    // last sample
    pub ns_last:i32,
    // radiance
    pub rad:Array3<f32>,
    pub rad_i16:Array3<i16>
    // // AVHRR radiance analysis
    // pub radanal:AvhrrRadAnal,
    // // IIS subgrid localization
    // pub iis_lon:[[f32;SGI];SNOT],
    // // IIS subgrid localization
    // pub iis_lat:[[f32;SGI];SNOT],
}

#[derive(Clone)]
pub struct EumAvhrr {
    pub clc:[[i8;PN];SNOT],
    pub lfr:[[i8;PN];SNOT],
    pub sif:[[i8;PN];SNOT]
}

#[derive(Debug)]
pub struct AvhrrRadAnal {
    pub channelid:[i32;NBK],
    pub nbclass:[[i32;PN];SNOT],
    pub wgt:[[[f32;NCL];PN];SNOT],
    pub y:[[[f32;NCL];PN];SNOT],
    pub z:[[[f32;NCL];PN];SNOT],
    pub mean:[[[[f32;NBK];NCL];PN];SNOT],
    pub std:[[[[f32;NBK];NCL];PN];SNOT],
    pub imageclassified:[[[i8;AMCO];AMLI];SNOT],
    pub imageclassifiednblin:[i16;SNOT],
    pub imageclassifiednbcol:[i16;SNOT],
    pub ccsmode:i8,
    pub classtype:[[i8;NCL];SNOT],
}

// 1-based IASI channel numbers
pub fn channel_of_nu(nu:f32)->usize {
    ((nu - NU0) / DNU).floor() as usize + 1
}

pub fn nu_of_channel(nu:usize)->f32 {
    NU0 + (nu - 1) as f32 * DNU
}

impl MdrL1C {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<Self> {
	// rec.seek_to_record(rd)?;
	let geps_iasi_mode = Self::l1c_get_iasi_mode(rd,rec)?;
	let geps_sp = Self::l1c_get_sp(rd,rec)?;
	let geps_ccd = Self::l1c_get_ccd(rd,rec)?;
	let (lon,lat) = Self::l1c_get_lon_lat(rd,rec)?;
	let (sza,saa) = Self::l1c_get_sun_angles(rd,rec)?;
	let (iza,iaa) = Self::l1c_get_metop_angles(rd,rec)?;
	let flg = Self::l1c_get_flag_qual_3(rd,rec)?;
	let cds_date = Self::l1c_get_dat_iasi(rd,rec)?;
	let EumAvhrr { clc,lfr,sif } = Self::l1c_get_eum_avhrr(rd,rec)?;
	let earth_sat_dist = Self::l1c_get_earth_sat_dist(rd,rec)?;
	Ok(Self {
	    geps_iasi_mode,
	    geps_sp,
	    geps_ccd,
	    lon,
	    lat,
	    flg,
	    cds_date,
	    sza,
	    saa,
	    iza,
	    iaa,
	    clc,
	    lfr,
	    sif,
	    earth_sat_dist
	})
    }

    fn l1c_get_earth_sat_dist<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<u32> {
	rec.seek_to_record(rd,276773)?;
	u32::read_bin(rd)
    }

    fn l1c_get_eum_avhrr<R:Read+Seek>(mut rd:&mut NatReader<R>,rec:&Grh)->
	Result<EumAvhrr>
    {
	rec.seek_to_record(rd,2728548)?;
	// XXX: Order
	let clc = <[[i8;PN];SNOT]>::read_bin(&mut rd)?;
	let lfr = <[[i8;PN];SNOT]>::read_bin(&mut rd)?;
	let sif = <[[i8;PN];SNOT]>::read_bin(&mut rd)?;
	Ok(EumAvhrr { clc,lfr,sif })
    }

    fn l1c_get_dat_iasi<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->
	Result<[ShortCdsTime;SNOT]>
    {
	rec.seek_to_record(rd,9122)?;
	<[ShortCdsTime;SNOT]>::read_bin(rd)
    }

    fn l1c_get_flag_qual_3<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->
	Result<[[[i8;SB];PN];SNOT]>
    {
	rec.seek_to_record(rd,255260)?;
	<[[[i8;SB];PN];SNOT]>::read_bin(rd)
    }

    fn l1c_get_iasi_mode<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<i32> {
	rec.seek_to_record(rd,24)?;
	let vali2_b = i16::read_bin(rd)?;
	let vali2 = vali2_b as i32;
	Ok(vali2.reverse_bits()) // XXX
    }

    fn l1c_get_sp<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<[i32;SNOT]> {
	rec.seek_to_record(rd,9380)?;
	<[i32;SNOT]>::read_bin(rd)
    }

    fn l1c_get_ccd<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<[i8;SNOT]> {
	rec.seek_to_record(rd,9350)?;
	<[i8;SNOT]>::read_bin(rd)
    }

    fn l1c_get_lon_lat<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->
	Result<Angles>
    {
	Self::l1c_get_angles_at(rd,rec,255893)
    }

    fn l1c_get_sun_angles<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->
	Result<Angles>
    {
	Self::l1c_get_angles_at(rd,rec,263813)
    }

    fn l1c_get_metop_angles<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->
	Result<Angles>
    {
	Self::l1c_get_angles_at(rd,rec,256853)
    }

    fn l1c_get_angles_at<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh,offset:u64)
				      ->Result<Angles>
    {
	rec.seek_to_record(rd,offset)?;
	let values = <[i32;2*PN*SNOT]>::read_bin(rd)?;
	let mut a = [[0.0;PN];SNOT];
	let mut b = [[0.0;PN];SNOT];
	for i in 0..SNOT {
	    for j in 0..PN {
		let k = i*PN + j;
		a[i][j] = values[2*k    ] as f32/1e6;
		b[i][j] = values[2*k + 1] as f32/1e6;
	    }
	}
	Ok((a,b))
    }
}

impl MdrL1CRad {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh,
				 giadr_sf:&GiadrScaleFactors)->Result<Self> {
	rec.seek_to_record(rd,276777)?;
	let d_wn : f32 = VInteger4::read_bin(rd)?.into();
	let d_wn = d_wn / 100.0;

	rec.seek_to_record(rd,276782)?;
	let ns_first = i32::read_bin(rd)?;
	let ns_last = i32::read_bin(rd)?;

	let mut values = vec![0;SS*PN*SNOT];
	for p_value in values.iter_mut() {
	    *p_value = i16::read_bin(rd)?;
	}

	let wn0 : f32 = d_wn * (ns_first - 1) as f32;

	let values_3d : Array3<i16> =
	    Array3::from_shape_vec((SNOT,PN,SS),values)?;
	let mut rad : Array3<f32> = Array3::zeros((SS,PN,SNOT));
	let mut rad_i16 : Array3<i16> = Array3::zeros((SS,PN,SNOT));
	for j in 0..SNOT {
	    for i in 0..PN {
		for jsf in 0..giadr_sf.i_def_scale_sond_nb_scale {
		    let powsf = 10.0_f32
			.powi(-giadr_sf.i_def_scale_sond_scale_factor
			      [jsf as usize] as i32);
		    for jc in
			(giadr_sf.i_def_scale_sond_ns_first
			 [jsf as usize] as i32) ..=
			(giadr_sf.i_def_scale_sond_ns_last
			 [jsf as usize] as i32).min(ns_last)
		    {
			let l = jc - ns_first;
			let v = values_3d[[j,i,l as usize]];
			rad_i16[[l as usize,i,j]] = v;
			rad[[l as usize,i,j]] = v as f32 * powsf;
		    }
		}
	    }
	}

	Ok(Self {
	    rad,
	    rad_i16,
	    wn0,
	    d_wn,
	    ns_first,
	    ns_last
	})
    }
}
