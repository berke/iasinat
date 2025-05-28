// Generic record header

use super::*;

#[derive(Clone,Copy,Default,Debug)]
pub struct ShortCdsTime {
    pub day:i16,
    pub msec:i32
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum GrhRecordClass {
    Reserved,
    Mphr,
    Sphr,
    Ipr,
    Geadr,
    Giadr,
    Veadr,
    Viadr,
    Mdr,
    Other(i8)
}

impl From<i8> for GrhRecordClass {
    fn from(c:i8)->Self {
	// EPS.GGS.SPE.96167 v8C 4.2.1.1
	use GrhRecordClass::*;

	match c {
	    0 => Reserved,
	    1 => Mphr,
	    2 => Sphr,
	    3 => Ipr,
	    4 => Geadr,
	    5 => Giadr,
	    6 => Veadr,
	    7 => Viadr,
	    8 => Mdr,
	    _ => Other(c)
	}
    }
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum GrhRecordKind {
    Mphr,
    GiadrQuality,
    GiadrScaleFactors,
    GiadrL1Eng,
    GiadrL2,
    ViadrEng,
    MdrL1C,
    MdrL1Eng,
    MdrL2,
    Other(i8,i8,i8)
}

impl Display for GrhRecordKind {
    fn fmt(&self,f:&mut Formatter<'_>)->Result<(),std::fmt::Error> {
	match self {
	    &Self::Other(cl,sc,vr) => {
		let cl : GrhRecordClass = cl.into();
		write!(f,"{:?}({},{})",cl,sc,vr)
	    }
	    _ => write!(f,"{:?}",self)
	}
    }
}

impl From<(i8,i8,i8)> for GrhRecordKind {
    fn from((c,s,v):(i8,i8,i8))->Self {
	match (c,s,v) {
	    (1,0,2) => Self::Mphr,
	    (5,0,_) => Self::GiadrQuality,
	    (5,1,2) => Self::GiadrScaleFactors,
	    (5,1,4) => Self::GiadrL2,
	    (5,2,_) => Self::GiadrL1Eng,
	    (7,0,_) => Self::ViadrEng,
	    (8,1,4) => Self::MdrL2,
	    (8,2,_) => Self::MdrL1C,
	    (8,3,_) => Self::MdrL1Eng,
	    _ => Self::Other(c,s,v)
	}
    }
}

#[derive(Debug)]
pub struct GiadrScaleFactors {
    pub i_def_scale_sond_nb_scale:i16,
    pub i_def_scale_sond_ns_first:[i16;10],
    pub i_def_scale_sond_ns_last:[i16;10],
    pub i_def_scale_sond_scale_factor:[i16;10],
    pub i_def_scale_iis_scale_scale_factor:i16
}

// EUM/OPS-EPS/MAN/04/0033 v3E p.60
#[derive(Debug)]
pub struct GiadrL2 {
    pub contents:GiadrL2Contents,
    // pub error_data:GiadrL2ErrorData,
    // pub brescia:GiadrL2Brescia,
    // pub forli:GiadrL2Forli,
}

// EUM/OPS-EPS/MAN/04/0033 v3E p.60
// Scaling factor applied
#[derive(Debug)]
pub struct GiadrL2Contents {
    /// Pressure levels on which retrieved temperature profiles
    /// are given [Pa]
    pub pressure_levels_temp:Vec<f64>,

    /// Pressure levels on which retrieved humidity profiles
    /// are given [Pa]
    pub pressure_levels_humidity:Vec<f64>,

    /// Pressure levels on which retrieved ozone profiles
    /// are given [Pa]
    pub pressure_levels_ozone:Vec<f64>,

    /// Wavelengths for surface emissivity [micron]
    pub surface_emissivity_wavelengths:Vec<f64>,
}

// EUM/OPS-EPS/MAN/04/0033 v3E p.60
#[derive(Debug)]
pub struct GiadrL2ErrorData {
    pub num_temperature_pcs:u8,
    pub num_water_vapour_pcs:u8,
    pub num_ozone_pcs:u8,
}

#[derive(Debug)]
pub struct MdrL2 {
    pub measurement_data:MdrL2MeasurementData,
    pub navigation_data_ifov:MdrL2NavigationDataIfov
}

#[derive(Debug)]
pub struct MdrL2MeasurementData {
    pub atmospheric_temperature:Array3<f64>,
    pub atmospheric_water_vapour:Array3<f64>,
    pub atmospheric_ozone:Array3<f64>,
    pub surface_temperature:Array2<f64>,
    pub surface_emissivity:Array3<f64>
}

#[derive(Debug)]
pub struct MdrL2NavigationDataIfov {
    pub angular_relation:Array3<f64>,
    pub earth_location:Array3<f64>,
}

pub const NU0 : f32 = 645.0;
pub const DNU : f32 = 0.25;
pub const NBR_IASI : usize = 8461;
pub const AMCO : usize = 100;
pub const AMLI : usize = 100;
pub const CCD : usize = 2;
pub const IMCO : usize = 64;
pub const IMLI : usize = 64;
pub const MAXBA : usize = 3600;
pub const NVP : usize = 221000;
pub const NIFVP : usize = 55000;
pub const NBK : usize = 6;
pub const NCL : usize = 7;
pub const NIM : usize = 28;
pub const PN : usize = 4;
pub const SB : usize = 3;
pub const SGI : usize = 25;
pub const SNOT : usize = 30;
pub const SNOT_P4 : usize = 34;
pub const SS : usize = 8700;
pub const VP : usize = 1;
pub const NLT : usize = 101;
pub const NLQ : usize = 101;
pub const NLO : usize = 101;
pub const NEW : usize = 12;
pub const NL_CO : usize = 19;
pub const NL_HNO3 : usize = 19;
pub const NL_O3 : usize = 40;
pub const NL_SO2 : usize = 5;
pub const NE : usize = 2048;
pub const NP : usize = 103;
pub const NSVERIF : usize = 4320;
pub const HUIT : usize = 8;
pub const DIX : usize = 10;

// 1-based IASI channel numbers
pub fn channel_of_nu(nu:f32)->usize {
    ((nu - NU0) / DNU).floor() as usize + 1
}

pub fn nu_of_channel(nu:usize)->f32 {
    NU0 + (nu - 1) as f32 * DNU
}

#[derive(Debug)]
pub struct VInteger4 {
    pub sf:i8,
    pub value:i32
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

#[derive(Debug)]
pub struct Mphr {
    pub product_name:String,
    pub parent_product_names:[Option<String>;4],
    pub spacecraft_id:String,
    pub sensing_start:Timestamp,
    pub sensing_end:Timestamp,
    pub state_vector_time:Timestamp,
    pub semi_major_axis:i64,
    pub eccentricity:i64,
    pub inclination:i64,
    pub perigee_argument:i64,
    pub right_ascension:i64,
    pub mean_anomaly:i64,
    pub x_position:i64,
    pub y_position:i64,
    pub z_position:i64,
    pub x_velocity:i64,
    pub y_velocity:i64,
    pub z_velocity:i64,
    pub earth_sun_distance_ratio:i64,
    pub location_tolerance_radial:i64,
    pub location_tolerance_crosstrack:i64,
    pub location_tolerance_alongtrack:i64,
    pub yaw_error:i64,
    pub roll_error:i64,
    pub pitch_error:i64,
    pub subsat_latitude_start:i64,
    pub subsat_longitude_start:i64,
    pub subsat_latitude_end:i64,
    pub subsat_longitude_end:i64,
    pub leap_second:i8,
    pub leap_second_utc:Option<Timestamp>,
    pub orbit_start:u32,
    pub orbit_end:u32,
}

#[derive(Debug)]
pub struct MdrL1C {
    // granule line number
    pub line:i32,
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
    pub rad:Array3<f32>
    // // AVHRR radiance analysis
    // pub radanal:AvhrrRadAnal,
    // // IIS subgrid localization
    // pub iis_lon:[[f32;SGI];SNOT],
    // // IIS subgrid localization
    // pub iis_lat:[[f32;SGI];SNOT],
}

pub struct GiadrL1Eng {
}

#[derive(Clone)]
pub struct EumAvhrr {
    pub clc:[[i8;PN];SNOT],
    pub lfr:[[i8;PN];SNOT],
    pub sif:[[i8;PN];SNOT]
}

pub type Angles = ([[f32;PN];SNOT],[[f32;PN];SNOT]);

pub type NatReader<R> = BufReader<R>;

impl GiadrScaleFactors {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<Self> {
	rec.seek_to_record(rd,20)?;
	let i_def_scale_sond_nb_scale = i16::read_bin(rd)?;
	let i_def_scale_sond_ns_first = <[i16;10]>::read_bin(rd)?;
	let i_def_scale_sond_ns_last = <[i16;10]>::read_bin(rd)?;
	let i_def_scale_sond_scale_factor = <[i16;10]>::read_bin(rd)?;
	let i_def_scale_iis_scale_scale_factor = i16::read_bin(rd)?;
	Ok(Self {
	    i_def_scale_sond_nb_scale,
	    i_def_scale_sond_ns_first,
	    i_def_scale_sond_ns_last,
	    i_def_scale_sond_scale_factor,
	    i_def_scale_iis_scale_scale_factor
	})
    }
}

pub const OMEGA_EARTH : f64 = 7.2921154e-5; // Earth ang. vel. [rad/s] - sidereal

impl Mphr {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<Self> {
	let product_name = Self::read_kv_string_at(
	    rd,rec,20,100,
	    "PRODUCT_NAME")?;
	let parent_product_name_1 = Self::read_opt_kv_string_at(
	    rd,rec,120,100,
	    "PARENT_PRODUCT_NAME_1")?;
	let parent_product_name_2 = Self::read_opt_kv_string_at(
	    rd,rec,220,100,
	    "PARENT_PRODUCT_NAME_2")?;
	let parent_product_name_3 = Self::read_opt_kv_string_at(
	    rd,rec,320,100,
	    "PARENT_PRODUCT_NAME_3")?;
	let parent_product_name_4 = Self::read_opt_kv_string_at(
	    rd,rec,420,100,
	    "PARENT_PRODUCT_NAME_4")?;
	let parent_product_names = [
	    parent_product_name_1,
	    parent_product_name_2,
	    parent_product_name_3,
	    parent_product_name_4
	];
	let spacecraft_id = Self::read_kv_string_at(
	    rd,rec,664,36,
	    "SPACECRAFT_ID")?.parse()?;

	let orbit_start : u32 = Self::read_kv_string_at(
	    rd,rec,1377,38,
	    "ORBIT_START")?.parse()?;
	let orbit_end : u32 = Self::read_kv_string_at(
	    rd,rec,1415,38,
	    "ORBIT_END")?.parse()?;
	let semi_major_axis : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1548,44,
		"SEMI_MAJOR_AXIS")?.parse()?;

	let tsp = TimestampParser::new()?;
	let sensing_start =
	    tsp.parse(&Self::read_kv_string_at(
		rd,rec,700,48,
		"SENSING_START")?)?;
	let sensing_end =
	    tsp.parse(&Self::read_kv_string_at(
		rd,rec,748,48,
		"SENSING_END")?)?;
	let state_vector_time =
	    tsp.parse(&Self::read_kv_string_at(
		rd,rec,1497,51,
		"STATE_VECTOR_TIME")?)?;
	let eccentricity : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1592,44,
		"ECCENTRICITY")?.parse()?;
	let inclination : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1636,44,
		"INCLINATION")?.parse()?;
	let perigee_argument : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1680,44,
		"PERIGEE_ARGUMENT")?.parse()?;
	let right_ascension : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1724,44,
		"RIGHT_ASCENSION")?.parse()?;
	let mean_anomaly : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1768,44,
		"MEAN_ANOMALY")?.parse()?;
	let x_position : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1812,44,
		"X_POSITION")?.parse()?;
	let y_position : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1856,44,
		"Y_POSITION")?.parse()?;
	let z_position : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1900,44,
		"Z_POSITION")?.parse()?;
	let x_velocity : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1944,44,
		"X_VELOCITY")?.parse()?;
	let y_velocity : i64 = 
	    Self::read_kv_string_at(
		rd,rec,1988,44,
		"Y_VELOCITY")?.parse()?;
	let z_velocity : i64 = 
	    Self::read_kv_string_at(
		rd,rec,2032,44,
		"Z_VELOCITY")?.parse()?;
	let earth_sun_distance_ratio : i64 = 
	    Self::read_kv_string_at(
		rd,rec,2076,44,
		"EARTH_SUN_DISTANCE_RATIO")?.parse()?;
	let location_tolerance_radial : i64 =
	    Self::read_kv_string_at(
		rd,rec,2120,44,
		"LOCATION_TOLERANCE_RADIAL")?.parse()?;
	let location_tolerance_crosstrack : i64 =
	    Self::read_kv_string_at(
		rd,rec,2164,44,
		"LOCATION_TOLERANCE_CROSSTRACK")?.parse()?;
	let location_tolerance_alongtrack : i64 =
	    Self::read_kv_string_at(
		rd,rec,2208,44,
		"LOCATION_TOLERANCE_ALONGTRACK")?.parse()?;
	let yaw_error : i64 =
	    Self::read_kv_string_at(
		rd,rec,2252,44,
		"YAW_ERROR")?.parse()?;
	let roll_error : i64 =
	    Self::read_kv_string_at(
		rd,rec,2296,44,
		"ROLL_ERROR")?.parse()?;
	let pitch_error : i64 =
	    Self::read_kv_string_at(
		rd,rec,2340,44,
		"PITCH_ERROR")?.parse()?;
	let subsat_latitude_start : i64 = 
	    Self::read_kv_string_at(
		rd,rec,2384,44,
		"SUBSAT_LATITUDE_START")?.parse()?;
	let subsat_longitude_start : i64 = 
	    Self::read_kv_string_at(
		rd,rec,2428,44,
		"SUBSAT_LONGITUDE_START")?.parse()?;
	let subsat_latitude_end : i64 = 
	    Self::read_kv_string_at(
		rd,rec,2472,44,
		"SUBSAT_LATITUDE_END")?.parse()?;
	let subsat_longitude_end : i64 = 
	    Self::read_kv_string_at(
		rd,rec,2516,44,
		"SUBSAT_LONGITUDE_END")?.parse()?;
	let leap_second : i8 = 
	    Self::read_kv_string_at(
		rd,rec,2560,35,
		"LEAP_SECOND")?.parse()?;
	let leap_second_utc =
	    if leap_second != 0 {
		Some(tsp.parse(&Self::read_kv_string_at(
		    rd,rec,2595,48,
		    "LEAP_SECOND_UTC")?)?)
	    } else {
		None
	    };

	Ok(Self {
	    product_name,
	    parent_product_names,
	    spacecraft_id,
	    sensing_start,
	    sensing_end,
	    state_vector_time,
	    semi_major_axis,
	    eccentricity,
	    inclination,
	    perigee_argument,
	    right_ascension,
	    mean_anomaly,
	    x_position,
	    y_position,
	    z_position,
	    x_velocity,
	    y_velocity,
	    z_velocity,
	    earth_sun_distance_ratio,
	    location_tolerance_radial,
	    location_tolerance_crosstrack,
	    location_tolerance_alongtrack,
	    yaw_error,
	    roll_error,
	    pitch_error,
	    subsat_latitude_start,
	    subsat_longitude_start,
	    subsat_latitude_end,
	    subsat_longitude_end,
	    leap_second,
	    leap_second_utc,
	    orbit_start,
	    orbit_end,
	})
    }

    pub fn read_kv_string_at<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh,
					  offset:u64,
					  size:usize,
					  name:&str)->
	Result<String>
    {
	Self::read_opt_kv_string_at(rd,rec,offset,size,name)?
	.ok_or_else(|| anyhow!("Missing value for {}",name))
    }

    pub fn read_opt_kv_string_at<R:Read+Seek>(rd:&mut NatReader<R>,
					      rec:&Grh,
					      offset:u64,
					      size:usize,
					      name:&str)->
	Result<Option<String>>
    {
	rec.seek_to_record(rd,offset)?;
	let mut u = vec![0;size];
	rd.read_exact(&mut u)?;
	let u = String::from_utf8_lossy(&u[..]);
	if let Some((v,w)) = u.trim().split_once('=') {
	    let v = v.trim();
	    if v == name {
		let w = w.trim();
		if w.chars().all(|c| c == 'x') {
		    Ok(None)
		} else {
		    Ok(Some(w.to_string()))
		}
	    } else {
		bail!("Unexpected key {:?}, was expecing {}",v,name);
	    }
	} else {
	    bail!("Invalid string {:?}",u);
	}
    }

    pub fn read_string_at<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh,
				       offset:u64,
				       size:usize)->
	Result<String>
    {
	rec.seek_to_record(rd,offset)?;
	let mut u = vec![0;size];
	rd.read_exact(&mut u)?;
	Ok(String::from_utf8_lossy(&u[..]).to_string())
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

	// XXX order
	let values_3d : Array3<i16> = Array3::from_shape_vec((SNOT,PN,SS),values)?;
	let mut rad : Array3<f32> = Array3::zeros((SS,PN,SNOT));
	for j in 0..SNOT {
	    for i in 0..PN {
		for jsf in 0..giadr_sf.i_def_scale_sond_nb_scale {
		    let powsf = 10.0_f32
			.powi(-giadr_sf.i_def_scale_sond_scale_factor
			      [jsf as usize] as i32);
		    for jc in
			(giadr_sf.i_def_scale_sond_ns_first
			 [jsf as usize] as i32)..
			(giadr_sf.i_def_scale_sond_ns_last
			 [jsf as usize] as i32).min(ns_last)
		    {
			let l = jc - ns_first;
			rad[[l as usize,i,j]] =
			    values_3d[[j,i,l as usize]]
			    as f32 * powsf;
		    }
		}
	    }
	}

	Ok(Self {
	    rad,
	    wn0,
	    d_wn,
	    ns_first,
	    ns_last
	})
    }
}

impl MdrL1C {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh,
				 line:i32)->Result<Self> {
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
	    line,
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

#[derive(Debug)]
pub struct Grh {
    pub record_kind:GrhRecordKind,
    pub instrument_group:i8,
    pub record_size:i32,
    pub record_start_time:ShortCdsTime,
    pub record_end_time:ShortCdsTime,
    pub record_pos:u64
}

impl VInteger4 {
    fn read_bin<R:Read>(mut rd:&mut NatReader<R>)->Result<Self> {
	let sf = i8::read_bin(&mut rd)?;
	let value = i32::read_bin(&mut rd)?;
	Ok(Self { sf,value })
    }
}

impl From<VInteger4> for f32 {
    fn from(v:VInteger4)->f32 {
	v.value as f32 / 10.0_f32.powi(v.sf as i32)
    }
}

impl ReadBinBig for ShortCdsTime {
    fn read_bin<R:Read>(rd:&mut R)->Result<Self> {
	let day = i16::read_bin(rd)?;
	let msec = i32::read_bin(rd)?;
	Ok(Self {
	    day,
	    msec
	})
    }
}

impl Grh {
    pub fn read_recs<R:Read+Seek>(rd:&mut NatReader<R>)->Result<Vec<Self>> {
	let mut recs = Vec::new();
	let len = rd.seek(SeekFrom::End(0))?;
	rd.seek(SeekFrom::Start(0))?;
	while rd.stream_position()? < len {
	    let grh = Self::read_bin(rd)?;
	    recs.push(grh);
	}
	Ok(recs)
    }

    pub fn scan_recs<R,T,F>(rd:&mut NatReader<R>,f:F)->Result<Option<T>>
    where
	R:Read+Seek,
	F:Fn(Self)->Result<Option<T>>
    {
	let len = rd.seek(SeekFrom::End(0))?;
	rd.seek(SeekFrom::Start(0))?;
	while rd.stream_position()? < len {
	    let grh = Self::read_bin(rd)?;
	    if let Some(x) = f(grh)? {
		return Ok(Some(x));
	    }
	}
	Ok(None)
    }

    pub fn seek_to_record<R:Read+Seek>(&self,rd:&mut NatReader<R>,offset:u64)->
	Result<()>
    {
	let pos = rd.stream_position()?;
	let target = self.record_pos + offset;
	let delta = target as i64 - pos as i64;
	rd.seek_relative(delta)?;
	Ok(())
    }

    pub fn read_bin<R:Read+Seek>(mut rd:&mut NatReader<R>)->Result<Self> {
	let record_pos = rd.stream_position()?;
	let class = i8::read_bin(&mut rd)?;
	let instrument_group = i8::read_bin(&mut rd)?;
	let subclass = i8::read_bin(&mut rd)?;
	let version = i8::read_bin(&mut rd)?;
	let record_size = i32::read_bin(&mut rd)?;
	let record_start_time = ShortCdsTime::read_bin(&mut rd)?;
	let record_end_time = ShortCdsTime::read_bin(&mut rd)?;
	let record_size_u64 : u64 = record_size.try_into()?;
	let record_kind : GrhRecordKind =
	    (class,subclass,version).into();
	rd.seek(SeekFrom::Start(record_pos + record_size_u64))?;
	Ok(Self {
	    record_kind,
	    instrument_group,
	    record_size,
	    record_start_time,
	    record_end_time,
	    record_pos
	})
    }
}

impl GiadrL2 {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<Self> {
	let contents = GiadrL2Contents::read_bin(rd,rec)?;
	Ok(Self {
	    contents
	})
    }
}

fn read_vec_map<R,T,U,F>(rd:&mut NatReader<R>,n:usize,mut f:F)->Result<Vec<U>>
where
    R:Read + Seek,
    T:ReadBinBig,
    F:FnMut(&T)->U
{
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
	let x = T::read_bin(rd)?;
	let y = f(&x);
	v.push(y);
    }
    Ok(v)
}

fn read_vec_and_scale<R:Read+Seek>(rd:&mut NatReader<R>,scale:f64)
				   ->Result<Vec<f64>>
{
    let n = u8::read_bin(rd)? as usize;
    read_vec_map(rd,n,|&x:&u32|->f64 { x as f64 / scale })
}

fn read_a2_map<R,T,U,F>(rd:&mut NatReader<R>,
			(d1,d2):(usize,usize),
			f:F)->Result<Array2<U>>
where
    R:Read + Seek,
    T:ReadBinBig,
    F:FnMut(&T)->U
{
    let v = read_vec_map(rd,d1*d2,f)?;
    let a : Array2<U> = Array2::from_shape_vec((d1,d2),v)?;
    Ok(a)
}

fn read_a3_map<R,T,U,F>(rd:&mut NatReader<R>,
			(d1,d2,d3):(usize,usize,usize),
			f:F)->Result<Array3<U>>
where
    R:Read + Seek,
    T:ReadBinBig,
    F:FnMut(&T)->U
{
    let v = read_vec_map(rd,d1*d2*d3,f)?;
    let a : Array3<U> = Array3::from_shape_vec((d1,d2,d3),v)?;
    Ok(a)
}

impl GiadrL2Contents {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<Self> {
	rec.seek_to_record(rd,20)?;
	let pressure_levels_temp = read_vec_and_scale(rd,1e2)?;
	rec.seek_to_record(rd,425)?;
	let pressure_levels_humidity = read_vec_and_scale(rd,1e2)?;
	rec.seek_to_record(rd,830)?;
	let pressure_levels_ozone = read_vec_and_scale(rd,1e2)?;
	rec.seek_to_record(rd,1235)?;
	let surface_emissivity_wavelengths = read_vec_and_scale(rd,1e4)?;
	Ok(Self {
	    pressure_levels_temp,
	    pressure_levels_humidity,
	    pressure_levels_ozone,
	    surface_emissivity_wavelengths
	})
    }
}

impl MdrL2 {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,giadr:&GiadrL2,rec:&Grh)
				 ->Result<Self>
    {
	let navigation_data_ifov =
	    MdrL2NavigationDataIfov::read_bin(rd,rec)?;
	let measurement_data =
	    MdrL2MeasurementData::read_bin(rd,giadr,rec)?;

	Ok(Self {
	    measurement_data,
	    navigation_data_ifov
	})
    }
}

fn u16_to_f64(x:u16,s:f64)->f64 {
    if x == u16::MAX {
	f64::NAN
    } else {
	x as f64 / s
    }
}

fn u32_to_f64(x:u32,s:f64)->f64 {
    if x == u32::MAX {
	f64::NAN
    } else {
	x as f64 / s
    }
}

fn i16_to_f64(x:i16,s:f64)->f64 {
    if x == i16::MAX {
	f64::NAN
    } else {
	x as f64 / s
    }
}

fn i32_to_f64(x:i32,s:f64)->f64 {
    if x == i32::MAX {
	f64::NAN
    } else {
	x as f64 / s
    }
}

impl MdrL2MeasurementData {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,giadr:&GiadrL2,rec:&Grh)
				 ->Result<Self>
    {
	rec.seek_to_record(rd,97702)?;
	let nlt = giadr.contents.pressure_levels_temp.len();
	let atmospheric_temperature =
	    read_a3_map(rd,(nlt,SNOT,PN),|&x| u16_to_f64(x,1e2))?; // XXX: Order

	rec.seek_to_record(rd,121942)?;
	let nlt = giadr.contents.pressure_levels_humidity.len();
	let atmospheric_water_vapour =
	    read_a3_map(rd,(nlt,SNOT,PN),|&x| u32_to_f64(x,1e7))?;

	rec.seek_to_record(rd,170422)?;
	let nlt = giadr.contents.pressure_levels_ozone.len();
	let atmospheric_ozone =
	    read_a3_map(rd,(nlt,SNOT,PN),|&x| u16_to_f64(x,1e8))?;

	rec.seek_to_record(rd,194662)?;
	let surface_temperature =
	    read_a2_map(rd,(SNOT,PN),|&x| u16_to_f64(x,1e2))?;

	rec.seek_to_record(rd,196342)?;
	let new = giadr.contents.surface_emissivity_wavelengths.len();
	let surface_emissivity =
	    read_a3_map(rd,(new,SNOT,PN),|&x| u16_to_f64(x,1e4))?;

	Ok(Self {
	    atmospheric_temperature,
	    atmospheric_water_vapour,
	    atmospheric_ozone,
	    surface_temperature,
	    surface_emissivity
	})
    }
}
impl MdrL2NavigationDataIfov {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)
				 ->Result<Self>
    {
	// rec.seek_to_record(rd,74321)?; // according to L2 PG
	rec.seek_to_record(rd,203067)?; // according to mod_l1c_l2_reading
	let angular_relation =
	    read_a3_map(rd,(PN,SNOT,4),|&x| i16_to_f64(x,1e2))?;

	rec.seek_to_record(rd,204027)?;
	let earth_location =
	    read_a3_map(rd,(PN,SNOT,2),|&x| i32_to_f64(x,1e4))?;
	Ok(Self {
	    angular_relation,
	    earth_location
	})
    }
}
