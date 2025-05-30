use super::*;

pub fn u16_to_f64(x:u16,s:f64)->f64 {
    if x == u16::MAX {
	f64::NAN
    } else {
	x as f64 / s
    }
}

pub fn u32_to_f64(x:u32,s:f64)->f64 {
    if x == u32::MAX {
	f64::NAN
    } else {
	x as f64 / s
    }
}

pub fn i16_to_f32(x:i16,s:f32)->f32 {
    if x == i16::MIN {
	f32::NAN
    } else {
	x as f32 / s
    }
}

pub fn i16_to_f64(x:i16,s:f64)->f64 {
    if x == i16::MIN {
	f64::NAN
    } else {
	x as f64 / s
    }
}

pub fn i32_to_f64(x:i32,s:f64)->f64 {
    if x == i32::MAX {
	f64::NAN
    } else {
	x as f64 / s
    }
}

pub fn read_vec_map<R,T,U,F>(rd:&mut NatReader<R>,n:usize,mut f:F)->Result<Vec<U>>
where
    R:Read + Seek,
    T:ReadBinBig,
    F:FnMut(&T)->U
{
    let mut v = Vec::with_capacity(n);
    for _ in 0..n {
	let x = T::read_bin(rd)?;
	let y = f(&x);
	v.push(y);
    }
    Ok(v)
}

pub fn read_vec_and_scale<R:Read+Seek>(rd:&mut NatReader<R>,scale:f64)
				   ->Result<Vec<f64>>
{
    let n = u8::read_bin(rd)? as usize;
    read_vec_map(rd,n,|&x:&u32|->f64 { x as f64 / scale })
}

pub fn read_a2_map<R,T,U,F>(rd:&mut NatReader<R>,
			(d1,d2):(usize,usize),
			f:F)->Result<Array2<U>>
where
    R:Read + Seek,
    T:ReadBinBig,
    F:FnMut(&T)->U
{
    let v = read_vec_map(rd,d1*d2,f)?;
    let a : Array2<U> = Array2::from_shape_vec((d1,d2),v)?;
    Ok(a)
}

pub fn read_a3_map<R,T,U,F>(rd:&mut NatReader<R>,
			(d1,d2,d3):(usize,usize,usize),
			f:F)->Result<Array3<U>>
where
    R:Read + Seek,
    T:ReadBinBig,
    F:FnMut(&T)->U
{
    let v = read_vec_map(rd,d1*d2*d3,f)?;
    let a : Array3<U> = Array3::from_shape_vec((d1,d2,d3),v)?;
    Ok(a)
}

#[derive(Debug)]
pub struct VInteger4 {
    pub sf:i8,
    pub value:i32
}

impl VInteger4 {
    pub fn read_bin<R:Read>(mut rd:&mut NatReader<R>)->Result<Self> {
	let sf = i8::read_bin(&mut rd)?;
	let value = i32::read_bin(&mut rd)?;
	Ok(Self { sf,value })
    }
}

impl From<VInteger4> for f32 {
    fn from(v:VInteger4)->f32 {
	v.value as f32 / 10.0_f32.powi(v.sf as i32)
    }
}
