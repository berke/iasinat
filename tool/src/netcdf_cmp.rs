use super::*;

pub struct NetcdfCmp {
    pub f1:nc::File,
    pub f2:nc::File,
    pub nbad_max:usize,
}

pub trait Comparator : Display {
    type T;

    fn init(&mut self,val1_var:&Variable,val2_var:&Variable)->Result<()>;
    fn add(&mut self,x1:Self::T,x2:Self::T)->bool;
    fn finish(&mut self);
    fn write_comparison<W:Write>(&self,i:usize,x1:Self::T,x2:Self::T,f:&mut W)
				 ->Result<(),std::fmt::Error>;
}

pub struct Comparison<'a,T,D> {
    pub name1:&'a str,
    pub name2:&'a str,
    pub ntot:usize,
    pub nbad:usize,
    pub bad:Vec<(usize,T,T)>,
    pub details:D
}

pub struct ComparatorF64 {
    pub s1:f64,
    pub s2:f64,
    pub e_max:f64,
    pub tol:f64,
}

pub struct ComparatorU8 {
    pub mask:u8
}

fn get_dim_1d<T>(v:&ArrayD<T>)->Result<usize> {
    if let &[m] = v.dim().slice() {
	Ok(m)
    } else {
	bail!("Array has wrong shape, expected 1 dimension")
    }
}

fn get_dim_2d<T>(v:&ArrayD<T>)->Result<(usize,usize)> {
    if let &[m,n] = v.dim().slice() {
	Ok((m,n))
    } else {
	bail!("Array has wrong shape, expected 2 dimensions")
    }
}

fn get_dim_3d<T>(v:&ArrayD<T>)->Result<(usize,usize,usize)> {
    if let &[m,n,o] = v.dim().slice() {
	Ok((m,n,o))
    } else {
	bail!("Array has wrong shape, expected 3 dimensions")
    }
}

fn get_dim_4d<T>(v:&ArrayD<T>)->Result<(usize,usize,usize,usize)> {
    if let &[m,n,o,p] = v.dim().slice() {
	Ok((m,n,o,p))
    } else {
	bail!("Array has wrong shape, expected 4 dimensions")
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

impl ComparatorF64 {
    pub fn new(tol:f64)->Self {
	Self {
	    s1:1.0,
	    s2:1.0,
	    e_max:0.0,
	    tol
	}
    }
}

impl Comparator for ComparatorF64 {
    type T = f64;

    fn init(&mut self,val1_var:&Variable,val2_var:&Variable)->Result<()> {
	self.s1 = get_scale_factor(val1_var,1.0)?;
	self.s2 = get_scale_factor(val2_var,1.0)?;
	Ok(())
    }

    fn add(&mut self,x1:Self::T,x2:Self::T)->bool {
	let e = (x1*self.s1 - x2*self.s2).abs();
	self.e_max = self.e_max.max(e);
	e > self.tol
    }
    
    fn finish(&mut self) {
    }

    fn write_comparison<W:Write>(&self,i:usize,x1:Self::T,x2:Self::T,f:&mut W)
				 ->Result<(),std::fmt::Error>
    {
	let x1 = x1*self.s1;
	let x2 = x2*self.s2;
	writeln!(f,"  {:8} {:12.6e} vs {:12.6e} (e={:.6e})",
		 i,
		 x1,
		 x2,
		 (x1 - x2).abs())?;
	Ok(())
    }
}

impl Display for ComparatorF64
{
    fn fmt(&self,f:&mut Formatter<'_>)->Result<(),std::fmt::Error> {
	write!(f,"e_max={:.6e}, tol={:.3e}",
	       self.e_max,
	       self.tol)
    }
}

impl ComparatorU8 {
    pub fn new(mask:u8)->Self {
	Self { mask }
    }
}

impl Comparator for ComparatorU8 {
    type T = u8;

    fn init(&mut self,_val1_var:&Variable,_val2_var:&Variable)->Result<()> {
	Ok(())
    }

    fn add(&mut self,x1:Self::T,x2:Self::T)->bool {
	(x1^x2) & self.mask != 0
    }
    
    fn finish(&mut self) {
    }

    fn write_comparison<W:Write>(&self,i:usize,x1:Self::T,x2:Self::T,f:&mut W)
				 ->Result<(),std::fmt::Error>
    {
	writeln!(f,"  {:8} {:3} (0x{:02x}) vs {:3} (0x{:02x})",
		 i,
		 x1,
		 x1,
		 x2,
		 x2)?;
	Ok(())
    }
}

impl Display for ComparatorU8
{
    fn fmt(&self,f:&mut Formatter<'_>)->Result<(),std::fmt::Error> {
	write!(f,"mask 0x{:02x}",self.mask)
    }
}


impl<T,C> Comparison<'_,T,C>
where
    T:Copy,
    C:Comparator<T=T>
{
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
	    info!("Comparing {} vs {}: OK, {}",
		  self.name1,
		  self.name2,
		  self.details);
	    Ok(())
	}
    }
}

impl<T,C> Display for Comparison<'_,T,C>
where
    T:Copy,
    C:Comparator<T=T>
{
    fn fmt(&self,f:&mut Formatter<'_>)->Result<(),std::fmt::Error> {
	writeln!(f,"total {}, bad {} ({:6.3}%), {}",
		 self.ntot,
		 self.nbad,
		 (self.nbad as f64 / self.ntot as f64)*100.0,
		 self.details)?;
	for &(i,x1,x2) in &self.bad {
	    self.details.write_comparison(i,x1,x2,f)?;
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

    fn compare_inner<'a,T,C,I1,I2>(
	&self,
	(name1,name2):(&'a str,&'a str),
	mut cmp:C,
	(iter1,iter2):(I1,I2))
	->Result<Comparison<'a,T,C>>
    where
	T:Copy,
	C:Comparator<T=T>,
	I1:Iterator<Item=T>,
	I2:Iterator<Item=T>,
    {
	let mut bad = Vec::new();
	let mut nbad = 0;
	let mut ntot = 0;
	for (i,(x1,x2)) in iter1.zip(iter2).enumerate() {
	    if cmp.add(x1,x2) {
		nbad += 1;
		if nbad < self.nbad_max {
		    bad.push((i,x1,x2));
		}
	    }
	    ntot += 1;
	}

	cmp.finish();

	Ok(Comparison {
	    name1,
	    name2,
	    ntot,
	    nbad,
	    bad,
	    details:cmp
	})
    }

    pub fn compare_1d<'a,T,C>(&self,name1:&'a str,name2:&'a str,mut cmp:C)
			      ->Result<Comparison<'a,T,C>>
    where
	T:Copy + nc::NcTypeDescriptor,
	C:Comparator<T=T>,
    {
	let val1_var = self.f1.variable(name1)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 1",name1))?;
	let val1 = val1_var.get::<T,_>(..)?;

	let val2_var = self.f2.variable(name2)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 2",name2))?;
	let val2 = val2_var.get::<T,_>(..)?;

	let m1 = get_dim_1d(&val1)?;
	let m2 = get_dim_1d(&val2)?;
	if m1 != m2 {
	    bail!("Dimension mismatch: {} vs {}",m1,m2);
	}

	cmp.init(&val1_var,&val2_var)?;

	self.compare_inner((name1,name2),
			   cmp,
			   (val1.iter().copied(),
			    val2.iter().copied()))
    }

    pub fn compare_2d_3d<'a,T,C>(&self,name1:&'a str,name2:&'a str,mut cmp:C)
			      ->Result<Comparison<'a,T,C>>
    where
	T:Copy + nc::NcTypeDescriptor,
	C:Comparator<T=T>,
    {
	let val1_var = self.f1.variable(name1)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 1",name1))?;
	let val1 = val1_var.get::<T,_>((..,..))?;

	let val2_var = self.f2.variable(name2)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 2",name2))?;
	let val2 = val2_var.get::<T,_>((..,..,..))?;

	let (nline1,nacross) = get_dim_2d(&val1)?;
	let (nline2,snot,pn) = get_dim_3d(&val2)?;

	if (nline1,nacross) != (nline2,snot*pn) {
	    bail!("Dimension mismatch: {},{} vs {},{}x{}",
		  nline1,nacross,
		  nline2,snot,pn);
	}

	cmp.init(&val1_var,&val2_var)?;

	self.compare_inner((name1,name2),
			   cmp,
			   (val1.iter().copied(),
			    val2.iter().copied()))
    }

    pub fn compare_3d_4d<'a,T,C>(&self,name1:&'a str,name2:&'a str,mut cmp:C)
			      ->Result<Comparison<'a,T,C>>
    where
	T:Copy + nc::NcTypeDescriptor,
	C:Comparator<T=T>,
    {
	let val1_var = self.f1.variable(name1)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 1",name1))?;
	let val1 = val1_var.get::<T,_>((..,..,..))?;

	let val2_var = self.f2.variable(name2)
	    .ok_or_else(|| anyhow!("Can't find variable {} in file 2",name2))?;
	let val2 = val2_var.get::<T,_>((..,..,..,..))?;

	let (nline1,nacross,n1) = get_dim_3d(&val1)?;
	let (nline2,snot,pn,n2) = get_dim_4d(&val2)?;

	if (nline1,nacross,n1) != (nline2,snot*pn,n2) {
	    bail!("Dimension mismatch: {},{},{} vs {},{}x{},{}",
		  nline1,nacross,n1,
		  nline2,snot,pn,n2);
	}

	cmp.init(&val1_var,&val2_var)?;

	self.compare_inner((name1,name2),
			   cmp,
			   (val1.iter().copied(),
			    val2.iter().copied()))
    }
}
