use std::io::{
    Read,
    Write
};

use super::*;

pub trait BinaryIo where Self:Sized {
    fn write_bin<W:Write>(&self,wr:&mut W)->Result<()>;
    fn read_bin<R:Read>(rd:&mut R)->Result<Self>;
}

pub trait BinaryIoBig where Self:Sized {
    fn write_bin<W:Write>(&self,wr:&mut W)->Result<()>;
    fn read_bin<R:Read>(rd:&mut R)->Result<Self>;
}

macro_rules! binary_io_implem_array {
    ($end:tt) => {
	impl<const N:usize,T> $end for [T;N] where Self:Sized,T:$end+Copy {
	    fn write_bin<W:Write>(&self,wr:&mut W)->Result<()>
	    {
		for x in self {
		    x.write_bin(wr)?;
		}
		Ok(())
	    }

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

macro_rules! binary_io_implem_gen {
    ($end:tt,$to:ident,$from:ident,$t:ty) => {
	impl $end for $t {
	    fn write_bin<W:Write>(&self,wr:&mut W)->Result<()> {
		let mut b = [0_u8;<$t>::BITS as usize / 8];
		b.copy_from_slice(&self.$to()[..]);
		wr.write_all(&b)?;
		Ok(())
	    }

	    fn read_bin<R:Read>(rd:&mut R)->Result<Self> {
		let mut b = [0_u8;<$t>::BITS as usize / 8];
		rd.read_exact(&mut b)?;
		Ok(<$t>::$from(b))
	    }
	}

    }
}

macro_rules! binary_io_implem {
    ($t:ty) => {
	binary_io_implem_gen!(BinaryIo,to_le_bytes,from_le_bytes,$t);
	binary_io_implem_gen!(BinaryIoBig,to_be_bytes,from_be_bytes,$t);
    }
}

binary_io_implem_array!(BinaryIo);
binary_io_implem_array!(BinaryIoBig);

binary_io_implem!(u8);
binary_io_implem!(u16);
binary_io_implem!(u32);
binary_io_implem!(u64);
binary_io_implem!(i8);
binary_io_implem!(i16);
binary_io_implem!(i32);
binary_io_implem!(i64);
