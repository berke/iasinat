use super::*;

#[derive(Debug)]
pub struct GiadrScaleFactors {
    pub i_def_scale_sond_nb_scale:i16,
    pub i_def_scale_sond_ns_first:[i16;10],
    pub i_def_scale_sond_ns_last:[i16;10],
    pub i_def_scale_sond_scale_factor:[i16;10],
    pub i_def_scale_iis_scale_scale_factor:i16
}

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
