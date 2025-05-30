use super::*;

#[derive(Debug)]
pub struct MdrL2 {
    pub measurement_data:MdrL2MeasurementData,
    pub navigation_data_scan_line:MdrL2NavigationDataScanLine,
    pub navigation_data_ifov:MdrL2NavigationDataIfov,
    pub processing_and_quality_flag:MdrL2ProcessingAndQualityFlag,
    pub first_guess_profiles:MdrL2FirstGuessProfiles,
    pub error_data:MdrL2ErrorData,
    pub forli_general:MdrL2ForliGeneral
}

#[derive(Debug)]
pub struct MdrL2MeasurementData {
    pub atmospheric_temperature:Array3<f32>,
    pub atmospheric_water_vapour:Array3<f64>,
    pub atmospheric_ozone:Array3<f32>,
    pub surface_temperature:Array2<f32>,
    pub surface_emissivity:Array3<f32>,
    pub integrated_water_vapour:Array2<f32>,
    pub integrated_ozone:Array2<f32>,
    pub integrated_n2o:Array2<f32>,
    pub integrated_co:Array2<f32>,
    pub integrated_ch4:Array2<f32>,
    pub integrated_co2:Array2<f32>,
    pub fractional_cloud_cover:Array3<f32>,
    pub surface_pressure:Array2<f64>,
}

#[derive(Debug)]
pub struct MdrL2NavigationDataScanLine {
    // pub time_attitude:u32,
    pub spacecraft_altitude:f32
}

#[derive(Debug)]
pub struct MdrL2NavigationDataIfov {
    pub angular_relation:Array3<f32>,
    pub earth_location:Array3<f64>,
}

#[derive(Debug)]
pub struct MdrL2ProcessingAndQualityFlag {
    pub flg_lansea:Array2<u8>
}


#[derive(Debug)]
pub struct MdrL2FirstGuessProfiles {
    pub fg_atmospheric_temperature:Array3<f32>,
    pub fg_atmospheric_water_vapour:Array3<f64>,
    pub fg_atmospheric_ozone:Array3<f32>,
    pub fg_surface_temperature:Array2<f32>,
}

#[derive(Debug)]
pub struct MdrL2ErrorData {
    pub temperature_error:Array3<f32>,
    pub water_vapour_error:Array3<f32>,
    pub ozone_error:Array3<f32>,
}

#[derive(Debug)]
pub struct MdrL2ForliGeneral {
    pub surface_z:Array2<f32>
}

impl MdrL2 {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh,giadr:&GiadrL2)
				 ->Result<Self>
    {
	let navigation_data_scan_line =
	    MdrL2NavigationDataScanLine::read_bin(rd,rec)?;
	let navigation_data_ifov =
	    MdrL2NavigationDataIfov::read_bin(rd,rec)?;
	let processing_and_quality_flag =
	    MdrL2ProcessingAndQualityFlag::read_bin(rd,rec)?;
	let measurement_data =
	    MdrL2MeasurementData::read_bin(rd,giadr,rec)?;
	let first_guess_profiles =
	    MdrL2FirstGuessProfiles::read_bin(rd,giadr,rec)?;
	let error_data =
	    MdrL2ErrorData::read_bin(rd,giadr,rec)?;
	let forli_general =
	    MdrL2ForliGeneral::read_bin(rd,rec)?;

	Ok(Self {
	    navigation_data_scan_line,
	    navigation_data_ifov,
	    processing_and_quality_flag,
	    measurement_data,
	    first_guess_profiles,
	    error_data,
	    forli_general
	})
    }
}

impl MdrL2MeasurementData {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,giadr:&GiadrL2,rec:&Grh)
				 ->Result<Self>
    {
	rec.seek_to_record(rd,97702)?;
	let nlt = giadr.contents.pressure_levels_temp.len();
	let atmospheric_temperature =
	    read_a3_map(rd,(SNOT,PN,nlt),|&x| u16_to_f32(x,1e2))?; // XXX: Order

	rec.seek_to_record(rd,121942)?;
	let nlt = giadr.contents.pressure_levels_humidity.len();
	let atmospheric_water_vapour =
	    read_a3_map(rd,(SNOT,PN,nlt),|&x| u32_to_f64(x,1e7))?;

	rec.seek_to_record(rd,170422)?;
	let nlt = giadr.contents.pressure_levels_ozone.len();
	let atmospheric_ozone =
	    read_a3_map(rd,(SNOT,PN,nlt),|&x| u16_to_f32(x,1e8))?;

	rec.seek_to_record(rd,194662)?;
	let surface_temperature =
	    read_a2_map(rd,(SNOT,PN),|&x| u16_to_f32(x,1e2))?;

	rec.seek_to_record(rd,194902)?;
	let integrated_water_vapour =
	    read_a2_map(rd,(SNOT,PN),|&x| u16_to_f32(x,1e2))?;

	rec.seek_to_record(rd,195142)?;
	let integrated_ozone =
	    read_a2_map(rd,(SNOT,PN),|&x| u16_to_f32(x,1e6))?;

	rec.seek_to_record(rd,195382)?;
	let integrated_n2o =
	    read_a2_map(rd,(SNOT,PN),|&x| u16_to_f32(x,1e6))?;

	rec.seek_to_record(rd,195622)?;
	let integrated_co =
	    read_a2_map(rd,(SNOT,PN),|&x| u16_to_f32(x,1e7))?;

	rec.seek_to_record(rd,195862)?;
	let integrated_ch4 =
	    read_a2_map(rd,(SNOT,PN),|&x| u16_to_f32(x,1e6))?;

	rec.seek_to_record(rd,196102)?;
	let integrated_co2 =
	    read_a2_map(rd,(SNOT,PN),|&x| u16_to_f32(x,1e3))?;

	rec.seek_to_record(rd,196342)?;
	let new = giadr.contents.surface_emissivity_wavelengths.len();
	let surface_emissivity =
	    read_a3_map(rd,(SNOT,PN,new),|&x| u16_to_f32(x,1e4))?;

	rec.seek_to_record(rd,199342)?;
	let fractional_cloud_cover =
	    read_a3_map(rd,(SNOT,PN,3),|&x| u16_to_f32(x,1e2))?;

	rec.seek_to_record(rd,202582)?;
	let surface_pressure =
	    read_a2_map(rd,(SNOT,PN),|&x| u32_to_f64(x,1.0))?;

	Ok(Self {
	    atmospheric_temperature,
	    atmospheric_water_vapour,
	    atmospheric_ozone,
	    surface_temperature,
	    surface_emissivity,
	    integrated_water_vapour,
	    integrated_ozone,
	    integrated_n2o,
	    integrated_co,
	    integrated_ch4,
	    integrated_co2,
	    fractional_cloud_cover,
	    surface_pressure
	})
    }
}

impl MdrL2NavigationDataScanLine {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)
				 ->Result<Self>
    {
	// rec.seek_to_record(rd,203049)?; // 74303?
	// let time_attitude = u32::read_bin(rd)?;

	rec.seek_to_record(rd,203063)?;
	let spacecraft_altitude = u32::read_bin(rd)? as f32 / 10.0;

	Ok(Self {
	    // time_attitude,
	    spacecraft_altitude
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
	    read_a3_map(rd,(SNOT,PN,4),|&x| i16_to_f32(x,1e2))?;

	rec.seek_to_record(rd,204027)?;
	let earth_location =
	    read_a3_map(rd,(SNOT,PN,2),|&x| i32_to_f64(x,1e4))?;
	Ok(Self {
	    angular_relation,
	    earth_location
	})
    }
}

impl MdrL2ProcessingAndQualityFlag {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)
				 ->Result<Self>
    {
	rec.seek_to_record(rd,206547)?;
	let flg_lansea = read_a2_map(rd,(SNOT,PN),|&x:&u8|->u8 { x })?;
	Ok(Self {
	    flg_lansea
	})
    }
}

impl MdrL2FirstGuessProfiles {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,giadr:&GiadrL2,rec:&Grh)
				 ->Result<Self>
    {
	rec.seek_to_record(rd,22)?;
	let nlt = giadr.contents.pressure_levels_temp.len();
	let fg_atmospheric_temperature =
	    read_a3_map(rd,(SNOT,PN,nlt),|&x| u16_to_f32(x,1e2))?; // XXX: Order

	rec.seek_to_record(rd,24262)?;
	let nlt = giadr.contents.pressure_levels_humidity.len();
	let fg_atmospheric_water_vapour =
	    read_a3_map(rd,(SNOT,PN,nlt),|&x| u32_to_f64(x,1e7))?;

	rec.seek_to_record(rd,72742)?;
	let nlt = giadr.contents.pressure_levels_ozone.len();
	let fg_atmospheric_ozone =
	    read_a3_map(rd,(SNOT,PN,nlt),|&x| u16_to_f32(x,1e8))?;

	rec.seek_to_record(rd,96982)?;
	let fg_surface_temperature =
	    read_a2_map(rd,(SNOT,PN),|&x| u16_to_f32(x,1e2))?;

	Ok(Self {
	    fg_atmospheric_temperature,
	    fg_atmospheric_water_vapour,
	    fg_atmospheric_ozone,
	    fg_surface_temperature,
	})
    }
}

impl MdrL2ErrorData {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,giadr:&GiadrL2,rec:&Grh)
				 ->Result<Self>
    {
	rec.seek_to_record(rd,207747)?;
	let nerr = u8::read_bin(rd)? as usize;

	rec.seek_to_record(rd,207748)?;
	let idx = read_a2_map(rd,(SNOT,PN),|&x:&u8|->u8 { x })?;

	let mut read_error = |nerrt:usize,offset:u64|->Result<Array3<f32>> {
	    rec.seek_to_record(rd,offset)?;

	    let err = read_a2_map(rd,(nerrt,nerr),|&x:&u32|->f32 {
		f32::from_bits(x) })?;

	    Ok(Array3::from_shape_fn(
		(SNOT,PN,nerrt),
		|(j,i,k)| {
		    let l = idx[[j,i]];
		    if l == u8::MAX {
			f32::NAN
		    } else {
			err[[k,l as usize]]
		    }
		}))
	};

	let temperature_error =
	    read_error(giadr.error_data.num_temperature_pcs as usize,207868)?;

	let water_vapour_error =
	    read_error(giadr.error_data.num_water_vapour_pcs as usize,256588)?;

	let ozone_error =
	    read_error(giadr.error_data.num_ozone_pcs as usize,277108)?;

	Ok(Self {
	    temperature_error,
	    water_vapour_error,
	    ozone_error
	})
    }
}

impl MdrL2ForliGeneral {
    pub fn read_bin<R:Read+Seek>(rd:&mut NatReader<R>,rec:&Grh)->Result<Self>
    {
	// rec.seek_to_record(rd,283708)?;
	rec.seek_to_record(rd,340828)?;
	let surface_z =
	    read_a2_map(rd,(SNOT,PN),|&x:&i16| i16_to_f32(x,1.0))?;
	Ok(Self {
	    surface_z
	})
    }
}
