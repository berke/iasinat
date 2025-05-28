use regex::Regex;

use super::*;

pub const JD_UNIX : f64 = 2440587.5;

pub struct TimestampParser {
    re:Regex,
}

pub trait ToUnix {
    fn to_unix(&self)->f64;
}

impl ToUnix for GregorianDateHMS { 
    fn to_unix(&self)->f64 {
	let (jd1,jd2) = self.to_julian();
	let t0 = ((jd1 - JD_UNIX) + jd2)*86400.0;
	t0
    }
}

pub fn julian_to_unix(jd1:R,jd2:R)->f64 {
    let t0 = ((jd1 - JD_UNIX) + jd2)*86400.0;
    t0
}

pub fn unix_to_julian(t:f64)->(R,R) {
    let t0 = t.floor();
    (t0/86400.0 + JD_UNIX,(t - t0)/86400.0)
}

pub fn now()->(R,R) {
    let dt = std::time::SystemTime::now()
	.duration_since(std::time::UNIX_EPOCH)
	.expect("Can't get timestamp");
    unix_to_julian(dt.as_secs_f64())
}
    
impl TimestampParser {
    pub fn new()->Result<Self> {
	let re = Regex::new(
	    r"^(\d{4})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2}|\d{5})Z$")?;
	Ok(Self { re })
    }

    pub fn parse(&self,u:&str)->Result<GregorianDateHMS> {
	let caps = self.re.captures(u)
	    .ok_or_else(|| anyhow!("Cannot parse timestamp from {:?}",u))?;
	Self::parse_caps(&caps,1)
    }

    fn parse_caps_gd(caps:&regex::Captures<'_>,i:usize)->
	Result<GregorianDate>
    {
	let year : i32 = caps.get(i    ).unwrap().as_str().parse()?;
	let month : i32 = caps.get(i + 1).unwrap().as_str().parse()?;
	let day : i32 = caps.get(i + 2).unwrap().as_str().parse()?;
	let gd = GregorianDate::new(year,month,day)?;
	Ok(gd)
    }

    fn parse_caps(caps:&regex::Captures<'_>,i:usize)->
	Result<GregorianDateHMS>
    {
	let date = Self::parse_caps_gd(caps,i)?;
	let hour : u8 = caps.get(i + 3).unwrap().as_str().parse()?;
	let minute : u8 = caps.get(i + 4).unwrap().as_str().parse()?;
	let second_s = caps.get(i + 5).unwrap().as_str();
	let second : u32 = second_s.parse()?;
	let second =
	    if second_s.len() == 2 {
		second as f64
	    } else {
		second as f64 / 1000.0
	    };
	let hms = HMS { hour,minute,second };
	Ok(Timestamp { date,hms })
    }
}

#[test]
fn test_timestamps() {
    let ts_unix =
	GregorianDateHMS {
	    date:GregorianDate::new(1970,1,1).unwrap(),
	    hms:HMS::new(0,0,0.0)
	};
    let (jd0_unix,jd1_unix) = ts_unix.to_julian();
    println!("UNIX epoch: JD {}",jd0_unix + jd1_unix);
    let ts_bill =
	GregorianDateHMS {
	    date:GregorianDate::new(2001,9,9).unwrap(),
	    hms:HMS::new(1,46,40.0)
	};
    let (jd0_bill,jd1_bill) = ts_bill.to_julian();
    println!("bill: JD {}",jd0_bill + jd1_bill);

    let t_bill = 86400.0*((jd0_bill - jd0_unix) + (jd1_bill - jd1_unix));
    println!("bill: Unix {:14.3}",t_bill);
    assert!((t_bill - 1e9).abs() < 1e-6);
}
