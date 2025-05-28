use super::*;

pub type Timestamp = GregorianDateHMS;

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
