use std::io::Read;

use super::*;

pub trait ReadBin where Self:Sized {
    fn read_bin<R:Read>(rd:&mut R)->Result<Self>;
}

pub trait ReadBinBig where Self:Sized {
    fn read_bin<R:Read>(rd:&mut R)->Result<Self>;

    fn read_bin_vec<R:Read>(rd:&mut R,n:usize)->Result<Vec<Self>> {
	let mut v = Vec::with_capacity(n);
	for _ in 0..n {
	    v.push(Self::read_bin(rd)?);
	}
	Ok(v)
    }
}

macro_rules! read_bin_implem_array {
    ($end:tt) => {
	impl<const N:usize,T> $end for [T;N] where Self:Sized,T:$end+Copy {
	    fn read_bin<R:Read>(rd:&mut R)->Result<Self>
	    {
		let x0 = T::read_bin(rd)?;
		let mut x = [x0;N];
		for i in 1..N {
		    x[i] = T::read_bin(rd)?;
		}
		Ok(x)
	    }
	}
    }
}

macro_rules! read_bin_implem_gen {
    ($end:tt,$to:ident,$from:ident,$t:ty) => {
	impl $end for $t {
	    fn read_bin<R:Read>(rd:&mut R)->Result<Self> {
		let mut b = [0_u8;<$t>::BITS as usize / 8];
		rd.read_exact(&mut b)?;
		Ok(<$t>::$from(b))
	    }
	}

    }
}

macro_rules! read_bin_implem {
    ($t:ty) => {
	read_bin_implem_gen!(ReadBin,to_le_bytes,from_le_bytes,$t);
	read_bin_implem_gen!(ReadBinBig,to_be_bytes,from_be_bytes,$t);
    }
}

read_bin_implem_array!(ReadBin);
read_bin_implem_array!(ReadBinBig);

read_bin_implem!(u8);
read_bin_implem!(u16);
read_bin_implem!(u32);
read_bin_implem!(u64);
read_bin_implem!(i8);
read_bin_implem!(i16);
read_bin_implem!(i32);
read_bin_implem!(i64);
