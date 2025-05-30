// Convenience functions

use super::*;

use std::{
    fs::File,
    path::Path,
    marker::PhantomData
};

pub struct LX<G,M> {
    br:BufReader<File>,
    recs:Vec<Grh>,
    irecs:Vec<usize>,
    giadr:G,
    mphr:Mphr,
    mk:PhantomData<M>
}

pub type L1C = LX<GiadrL1C,MdrL1C>;
pub type L2 = LX<GiadrL2,MdrL2>;

impl<G,M> LX<G,M>
where
    M:Level<G>
{
    pub fn open<P:AsRef<Path>>(path:P)->Result<Self> {
	let fd = File::open(path)?;
	let mut br = BufReader::new(fd);
	let recs = Grh::read_recs(&mut br)?;
	let mut mphr = None;
	let mut giadr = None;
	let mut irecs = Vec::with_capacity(recs.len());
	for (irec,rec) in recs.iter().enumerate() {
	    let kind = &rec.record_kind;
	    match kind {
		GrhRecordKind::Mphr =>
		    mphr = Some(Mphr::read_bin(&mut br,rec)?),
		_ => {
		    match M::classify_record(&rec.record_kind) {
			RecordClassification::Giadr =>
			    giadr = Some(M::read_giadr(&mut br,rec)?),
			RecordClassification::Mdr => irecs.push(irec),
			RecordClassification::Other => ()
		    }
		}
	    }
	}
	if let (Some(mphr),Some(giadr)) = (mphr,giadr) {
	    Ok(Self {
		br,
		recs,
		irecs,
		giadr,
		mphr,
		mk:PhantomData
	    })
	} else {
	    bail!("Cannot find GIADR or MPHR");
	}
    }

    pub fn nline(&self)->usize {
	self.irecs.len()
    }

    pub fn read_line(&mut self,iline:usize)->Result<M> {
	let irec = self.irecs[iline];
	let rec = &self.recs[irec];
	let l = M::read_mdr(&mut self.br,rec,&self.giadr)?;
	Ok(l)
    }

    pub fn mphr(&self)->&Mphr {
	&self.mphr
    }

    pub fn giadr(&self)->&G {
	&self.giadr
    }
}
