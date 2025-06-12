use super::*;

pub struct NetcdfCmp {
    f1:nc::File,
    f2:nc::File,
    nbad_max:usize,
}

fn get_dim_1d<T>(v:&ArrayD<T>)->Result<usize> {
    if let &[m] = v.dim().slice() {
	Ok(m)
    } else {
	bail!("Array has wrong shape")
    }
}

fn get_dim_2d<T>(v:&ArrayD<T>)->Result<(usize,usize)> {
    if let &[m,n] = v.dim().slice() {
	Ok((m,n))
    } else {
	bail!("Array has wrong shape")
    }
}

fn get_dim_3d<T>(v:&ArrayD<T>)->Result<(usize,usize,usize)> {
    if let &[m,n,o] = v.dim().slice() {
	Ok((m,n,o))
    } else {
	bail!("Array has wrong shape")
    }
}

fn get_dim_4d<T>(v:&ArrayD<T>)->Result<(usize,usize,usize,usize)> {
    if let &[m,n,o,p] = v.dim().slice() {
	Ok((m,n,o,p))
    } else {
	bail!("Array has wrong shape")
    }
}

fn get_scale_factor<T>(v:&Variable,default:T)->Result<T>
where
    T:TryFrom<AttributeValue>,
    <T as TryFrom<AttributeValue>>::Error : std::error::Error + Send + Sync + 'static
{
    if let Some(a) = v.attribute("scale_factor") {
	let val = a.value()?;
	let x : T = val.try_into()?;
	Ok(x)
    } else {
	Ok(default)
    }
}

pub struct NetcdfCmpResult<'a> {
    pub name1:&'a str,
    pub name2:&'a str,
    pub ntot:usize,
    pub nbad:usize,
    pub bad:Vec<(usize,f64,f64)>,
    pub e_max:f64,
    pub tol:f64,
}

impl NetcdfCmpResult<'_> {
    pub fn good(&self)->bool {
	self.nbad == 0
    }

    pub fn check(&self)->Result<()> {
	if !self.good() {
	    error!("Mismatch in {} vs {}\n{}",
		   self.name1,
		   self.name2,
		   self);
	    bail!("Data mismatch");
	} else {
	    info!("Comparing {} vs {}: OK, e_max={:.6e}, tol={:.3e}",
		  self.name1,
		  self.name2,
		  self.e_max,
		  self.tol);
	    Ok(())
	}
    }
}

impl Display for NetcdfCmpResult<'_> {
    fn fmt(&self,f:&mut Formatter<'_>)->Result<(),std::fmt::Error> {
	writeln!(f,"total {}, bad {} ({:6.3}%), e_max {:.6e}",
		 self.ntot,
		 self.nbad,
		 (self.nbad as f64 / self.ntot as f64)*100.0,
		 self.e_max)?;
	for (i,x1,x2) in &self.bad {
	    writeln!(f,"  {:8} {:12.6e} vs {:12.6e} (e={:.6e})",
		     i,
		     x1,
		     x2,
		     (x1 - x2).abs())?;
	}
	if self.bad.len() < self.nbad {
	    writeln!(f,"...")?;
	}
	Ok(())
    }
}

impl NetcdfCmp {
    pub fn new(f1:nc::File,f2:nc::File)->Result<Self> {
	Ok(Self {
	    f1,
	    f2,
	    nbad_max:10
	})
    }

    fn compare_inner<'a,I1,I2>(
	&self,
	(name1,name2):(&'a str,&'a str),
	tol:f64,
	(val1_var,val2_var):(&Variable,&Variable),
	(iter1,iter2):(I1,I2)
    )->
	Result<NetcdfCmpResult<'a>>
    where
	I1:Iterator<Item=f64>,
	I2:Iterator<Item=f64>,
    {
	// Get scale factors
	let s1 = get_scale_factor(val1_var,1.0)?;
	let s2 = get_scale_factor(val2_var,1.0)?;

	let mut e_max : f64 = 0.0;
	let mut bad = Vec::new();
	let mut nbad = 0;
	let mut ntot = 0;
	for (i,(x1,x2)) in iter1.zip(iter2).enumerate() {
	    let x1 = x1*s1;
	    let x2 = x2*s2;
	    let e = (x1 - x2).abs();
	    if e > tol {
		nbad += 1;
		if nbad < self.nbad_max {
		    bad.push((i,x1,x2));
		}
	    }
	    e_max = e_max.max(e);
	    ntot += 1;
	}

	Ok(NetcdfCmpResult {
	    name1,
	    name2,
	    ntot,
	    nbad,
	    bad,
	    e_max,
	    tol
	})
    }


    pub fn compare_1d<'a>(&self,name1:&'a str,name2:&'a str,tol:f64)->
	Result<NetcdfCmpResult<'a>>
    {
	let val1_var = self.f1.variable(name1)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 1",name1))?;
	let val1 = val1_var.get::<f64,_>(..)?;

	let val2_var = self.f2.variable(name2)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 2",name2))?;
	let val2 = val2_var.get::<f64,_>(..)?;

	let m1 = get_dim_1d(&val1)?;
	let m2 = get_dim_1d(&val2)?;
	if m1 != m2 {
	    bail!("Dimension mismatch: {} vs {}",m1,m2);
	}

	self.compare_inner((name1,name2),
			   tol,
			   (&val1_var,&val2_var),
			   (val1.iter().copied(),
			    val2.iter().copied()))
    }

    pub fn compare_2d_3d<'a>(&self,name1:&'a str,name2:&'a str,tol:f64)->
	Result<NetcdfCmpResult<'a>>
    {
	let val1_var = self.f1.variable(name1)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 1",name1))?;
	let val1 = val1_var.get::<f64,_>((..,..))?;

	let val2_var = self.f2.variable(name2)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 2",name2))?;
	let val2 = val2_var.get::<f64,_>((..,..,..))?;

	let (nline1,nacross) = get_dim_2d(&val1)?;
	let (nline2,snot,pn) = get_dim_3d(&val2)?;

	if (nline1,nacross) != (nline2,snot*pn) {
	    bail!("Dimension mismatch: {},{} vs {},{}x{}",
		  nline1,nacross,
		  nline2,snot,pn);
	}

	self.compare_inner((name1,name2),
			   tol,
			   (&val1_var,&val2_var),
			   (val1.iter().copied(),
			    val2.iter().copied()))
    }

    pub fn compare_3d_4d<'a>(&self,name1:&'a str,name2:&'a str,tol:f64)->
	Result<NetcdfCmpResult<'a>>
    {
	let val1_var = self.f1.variable(name1)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 1",name1))?;
	let val1 = val1_var.get::<f64,_>((..,..,..))?;

	let val2_var = self.f2.variable(name2)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 2",name2))?;
	let val2 = val2_var.get::<f64,_>((..,..,..,..))?;

	let (nline1,nacross,n1) = get_dim_3d(&val1)?;
	let (nline2,snot,pn,n2) = get_dim_4d(&val2)?;

	if (nline1,nacross,n1) != (nline2,snot*pn,n2) {
	    bail!("Dimension mismatch: {},{},{} vs {},{}x{},{}",
		  nline1,nacross,n1,
		  nline2,snot,pn,n2);
	}

	self.compare_inner((name1,name2),
			   tol,
			   (&val1_var,&val2_var),
			   (val1.iter().copied(),
			    val2.iter().copied()))
    }
}
