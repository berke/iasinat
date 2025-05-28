use super::*;

use tofas::{
    common::R,
    calendar::GregorianDateHMS,
};

#[derive(Clone,Copy,Default,Debug)]
pub struct ShortCdsTime {
    pub day:i16,
    pub msec:i32
}

pub const JD_UNIX : R = 2440587.5;

pub const JD2000 : R = 2451545.5;

impl ShortCdsTime {
    pub fn to_julian(&self)->(R,R) {
	(JD2000 + self.day as f64,
	 self.msec as f64 / 86400000.0)
    }

    pub fn to_gregorian_hms(&self)->Result<GregorianDateHMS> {
	let (jd1,jd2) = self.to_julian();
	Ok(GregorianDateHMS::from_julian(jd1,jd2)?)
    }

    pub fn to_unix(&self)->R {
	let (jd1,jd2) = self.to_julian();
	timestamp::julian_to_unix(jd1,jd2)
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
