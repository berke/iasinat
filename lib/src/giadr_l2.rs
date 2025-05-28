use super::*;

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

impl GiadrL2 {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<Self> {
	let contents = GiadrL2Contents::read_bin(rd,rec)?;
	Ok(Self {
	    contents
	})
    }
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
