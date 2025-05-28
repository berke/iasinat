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

const JD_UNIX : (R,R) = (2440588.5,0.0);
const JD2000 : (R,R) = (2451545.5,0.0);

impl ShortCdsTime {
    pub fn to_julian(&self)->(R,R) {
	let (jd2000_1,jd2000_2) = JD2000;
	(jd2000_1 + self.day as f64,
	 jd2000_2 + self.msec as f64 / 86400000.0)
    }

    pub fn to_gregorian_hms(&self)->Result<GregorianDateHMS> {
	let (jd1,jd2) = self.to_julian();
	Ok(GregorianDateHMS::from_julian(jd1,jd2)?)
    }

    pub fn to_unix(&self)->R {
	let (jd_unix_1,jd_unix_2) = JD_UNIX;
	let (jd1,jd2) = self.to_julian();
	let t0 = ((jd1 - jd_unix_1) + (jd2 - jd_unix_2))*86400.0;
	t0
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
