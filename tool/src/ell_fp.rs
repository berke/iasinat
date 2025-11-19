use super::*;

use circfp::{
    ObservationAngles
};

#[derive(Clone,Debug)]
pub struct PixelInfo {
    pub time_range:(f64,f64),
    pub angles:ObservationAngles,
    pub height:f64,
}

pub struct EllFpProcessor {
    params:bool,
    points:usize,
    hca:f64,
    #[cfg(feature="footprints-mpk")]
    mpk_opts:EllFpMpkOptions
}

pub const IFPCOORD_LON : usize = 0;
pub const IFPCOORD_LAT : usize = 1;
pub const NFPCOORD : usize = 2;

pub const IFPELL_A : usize = 0;
pub const IFPELL_B : usize = 1;
pub const IFPELL_PA : usize = 2;
pub const NFPELL : usize = 3;

const DEGREE : f64 = std::f64::consts::PI/180.0;

pub struct EllFps {
    #[allow(unused)]
    pub times:Array3<(f64,f64)>,

    #[allow(unused)]
    pub coords:Array4<f64>,

    pub ells:Array4<f32>,
    pub lats:Array4<f32>,
    pub lons:Array4<f32>
}

pub const HELP : Seq<'static,&'static str> = Seq::Cat(&[
    &Seq::One(&"\
Footprint generation
====================
--fp-params  Add footprint geometries (ellipse parameters)
--fp-points  N
             Add footprint polygons (sample ellipses at N points)
--hca-ifov   RADIANS
	     Half cone-angle of the iFOVs"),
    #[cfg(feature="footprints-mpk")]
    &ell_fp_mpk::HELP
]);

impl EllFpProcessor {
    pub fn from_args(args:&mut Arguments)->Result<Self> {
	let params = args.contains("--fp-params");
	let points : usize = args.opt_value_from_str("--fp-points")?
	    .unwrap_or(0);
	let hca = args.opt_value_from_str("--hca-ifov")?.unwrap_or(HCA_IFOV);

	#[cfg(feature="footprints-mpk")]
	let mpk_opts = EllFpMpkOptions::from_args(args)?;

	Ok(Self {
	    params,
	    points,
	    hca,

	    #[cfg(feature="footprints-mpk")]
	    mpk_opts
	})
    }

    pub fn active(&self)->bool {
	#[cfg(feature="footprints-mpk")]
	let x = self.mpk_opts.active();
	#[cfg(not(feature="footprints-mpk"))]
	let x = false;

	x || self.params || self.points > 0
    }

    pub fn compute<F>(&self,nline:usize,mut pixel:F)->Result<EllFps>
    where
	F:FnMut(usize,usize,usize)->PixelInfo
    {
	trace!("Computing footprints");
	let geo = EllipsoidConverter::new(&WGS84)?;
	let mut coords : Array4<f64> = Array4::zeros((nline,SNOT,PN,NFPCOORD));
	let mut times : Array3<(f64,f64)> = Array3::from_elem((nline,SNOT,PN),
							      (0.0,0.0));
	let mut ells : Array4<f32> = Array4::zeros((nline,SNOT,PN,NFPELL));
	let mut lats : Array4<f32> = Array4::zeros((nline,SNOT,PN,self.points));
	let mut lons : Array4<f32> = Array4::zeros((nline,SNOT,PN,self.points));
	for iline in 0..nline {
	    for j in 0..SNOT {
		for i in 0..PN {
		    let PixelInfo {
			angles,
			height,
			time_range
		    } = pixel(iline,j,i);
		    coords[[iline,j,i,IFPCOORD_LON]] = angles.lon;
		    coords[[iline,j,i,IFPCOORD_LAT]] = angles.lat;
		    times[[iline,j,i]] = time_range;
		    if let Ok(obs) = geo.estimate_observation(&angles,height)
			&& let Ok(fp) = geo.estimate_footprint(&obs,self.hca)
		    {
			ells[[iline,j,i,IFPELL_A]] = (fp.a/1e3) as f32;
			ells[[iline,j,i,IFPELL_B]] = (fp.b/1e3) as f32;
			ells[[iline,j,i,IFPELL_PA]] = (fp.pa/DEGREE) as f32;
			if self.points > 0 {
			    let ol = fp.outline(self.points)?;
			    for (k,&p) in ol.iter().enumerate() {
				let p : [f64;3] = p.into();
				let gd : Geodetic360 =
				    geo.geocentric_to_geodetic(&p).into();
				lats[[iline,j,i,k]] = gd.lat as f32;
				lons[[iline,j,i,k]] = gd.lon as f32;
			    }
			}
		    }
		}
	    }
	}
	Ok(EllFps {
	    times,
	    coords,
	    ells,
	    lats,
	    lons
	})
    }

    pub fn add_to_dataset(&self,fd_out:&mut FileMut,fps:&EllFps)->Result<()> {
	if self.params {
	    trace!("Adding footprint ellipses");
	    fd_out.add_dimension("fpell",NFPELL)?;

	    let mut var = fd_out.add_variable::<f32>(
		"fp_ell",
		&["line","snot","pn","fpell"])?;
	    var.set_fill_value(f32::NAN)?;
	    var.put(fps.ells.view(),(..,..,..,..))?;
	    var.put_attribute("long_name",
			      "footprint ellipse parameters (a,b,pa)")?;
	    var.put_attribute("units","km,km,degrees_north")?;
	}

	if self.points > 0 {
	    trace!("Adding footprint polygons");
	    fd_out.add_dimension("fpvertex",self.points)?;

	    let mut var = fd_out.add_variable::<f32>(
		"fp_lat",
		&["line","snot","pn","fpvertex"])?;
	    var.set_fill_value(f32::NAN)?;
	    var.put(fps.lats.view(),(..,..,..,..))?;
	    var.put_attribute("long_name",
			      "footprint vertex latitude")?;
	    var.put_attribute("units","degrees_north")?;

	    let mut var = fd_out.add_variable::<f32>(
		"fp_lon",
		&["line","snot","pn","fpvertex"])?;
	    var.set_fill_value(f32::NAN)?;
	    var.put(fps.lons.view(),(..,..,..,..))?;
	    var.put_attribute("long_name",
			      "footprint vertex longitude")?;
	    var.put_attribute("units","degrees_east")?;
	}

	Ok(())
    }

    #[cfg(feature="footprints-mpk")]
    pub fn save_mpk(&self,fps:&EllFps,mphr:&Mphr)->Result<()> {
	fps.save_mpk(mphr,&self.mpk_opts)
    }
}
