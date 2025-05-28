use super::*;

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

#[derive(Debug)]
pub struct Grh {
    pub record_kind:GrhRecordKind,
    pub instrument_group:i8,
    pub record_size:i32,
    pub record_start_time:ShortCdsTime,
    pub record_end_time:ShortCdsTime,
    pub record_pos:u64
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
