// Convenience functions

use super::*;

use std::{
    fs::File,
    path::Path
};

pub struct L1C {
    br:BufReader<File>,
    recs:Vec<Grh>,
    l1c_irecs:Vec<usize>,
    sf:GiadrScaleFactors,
    mphr:Mphr
}

impl L1C {
    pub fn open<P:AsRef<Path>>(path:P)->Result<Self> {
	let fd = File::open(path)?;
	let mut br = BufReader::new(fd);
	let recs = Grh::read_recs(&mut br)?;
	let mut mphr = None;
	let mut sf = None;
	let mut l1c_irecs = Vec::with_capacity(recs.len());
	for (irec,rec) in recs.iter().enumerate() {
	    match rec.record_kind {
		GrhRecordKind::MdrL1C => l1c_irecs.push(irec),
		GrhRecordKind::GiadrScaleFactors =>
		    sf = Some(GiadrScaleFactors::read_bin(&mut br,rec)?),
		GrhRecordKind::Mphr =>
		    mphr = Some(Mphr::read_bin(&mut br,rec)?),
		_ => ()
	    }
	}
	if let (Some(mphr),Some(sf)) = (mphr,sf) {
	    Ok(Self {
		br,
		recs,
		l1c_irecs,
		sf,
		mphr
	    })
	} else {
	    bail!("Cannot find GIADR or MPHR");
	}
    }

    pub fn nline(&self)->usize {
	self.l1c_irecs.len()
    }

    pub fn read_l1c(&mut self,iline:usize)->Result<MdrL1C> {
	let irec = self.l1c_irecs[iline];
	let rec = &self.recs[irec];
	let l1c = MdrL1C::read_bin(&mut self.br,rec)?;
	Ok(l1c)
    }

    pub fn read_l1c_rad(&mut self,iline:usize)->Result<MdrL1CRad> {
	let irec = self.l1c_irecs[iline];
	let rec = &self.recs[irec];
	let l1c = MdrL1CRad::read_bin(&mut self.br,rec,&self.sf)?;
	Ok(l1c)
    }

    pub fn mphr(&self)->&Mphr {
	&self.mphr
    }

    pub fn giadr(&self)->&GiadrScaleFactors {
	&self.sf
    }
}
