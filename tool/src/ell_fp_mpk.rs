use super::*;

use footprint::{
    Footprint,
    Footprints
};

pub trait EllFpMpk {
    fn save_mpk<P:AsRef<Path>>(&self,path:P)->Result<()>;
}

impl EllFpMpk for EllFps {
    fn save_mpk<P:AsRef<Path>>(&self,path:P)->Result<()> {
	let mut footprints = Vec::new();
	let fps = Footprints{ footprints };
	fps.save_to_file(path)?;
	Ok(())
    }
}
