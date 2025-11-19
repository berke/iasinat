use super::*;

use circfp::{
    ObservationAngles
};

pub struct EllFpProcessor {
    params:bool,
    points:usize,
    hca:f64,
    #[cfg(feature="footprints-mpk")]
    mpk:Option<OsString>
}

const IFPELL_A : usize = 0;
const IFPELL_B : usize = 1;
const IFPELL_PA : usize = 2;
const NFPELL : usize = 3;

const DEGREE : f64 = std::f64::consts::PI/180.0;

pub struct EllFps {
    ells:Array4<f32>,
    lats:Array4<f32>,
    lons:Array4<f32>
}

pub const HELP : Seq<'static,&'static str> = Seq::Cat(&[
    &Seq::One(&"\n\
EllFp generation
--------------------
--fp-params  Add footprint geometries (ellipse parameters)
--fp-points  N
             Add footprint polygons (sample ellipses at N points)
--hca-ifov   RADIANS
	     Half cone-angle of the iFOVs"),
    #[cfg(feature="footprints-mpk")]
    &Seq::One(&"\
	--mpk PATH   Save footprints in MPK footprint format")
]);

impl EllFpProcessor {
    pub fn from_args(args:&mut Arguments)->Result<Self> {
	let params = args.contains("--fp-params");
	let points : usize = args.opt_value_from_str("--fp-points")?
	    .unwrap_or(0);
	let hca = args.opt_value_from_str("--hca-ifov")?.unwrap_or(HCA_IFOV);
	let mpk : Option<OsString> = args.opt_value_from_str("--mpk")?;
	Ok(Self {
	    params,
	    points,
	    hca,

	    #[cfg(feature="footprints-mpk")]
	    mpk
	})
    }

    pub fn active(&self)->bool {
	#[cfg(feature="footprints-mpk")]
	let x = self.mpk.is_some();
	#[cfg(not(feature="footprints-mpk"))]
	let x = false;

	x || self.params || self.points > 0
    }

    pub fn compute<F>(&self,nline:usize,mut pixel:F)->Result<EllFps>
    where
	F:FnMut(usize,usize,usize)->(ObservationAngles,f64)
    {
	trace!("Computing footprints");
	let geo = EllipsoidConverter::new(&WGS84)?;
	let mut ells : Array4<f32> = Array4::zeros((nline,SNOT,PN,NFPELL));
	let mut lats : Array4<f32> = Array4::zeros((nline,SNOT,PN,self.points));
	let mut lons : Array4<f32> = Array4::zeros((nline,SNOT,PN,self.points));
	for iline in 0..nline {
	    for j in 0..SNOT {
		for i in 0..PN {
		    let (angles,height) = pixel(iline,j,i);
		    if let Ok(obs) = geo.estimate_observation(&angles,height)
		    {
			if let Ok(fp) = geo.estimate_footprint(&obs,self.hca) {
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
	}
	Ok(EllFps {
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
    pub fn save_mpk(&self,fps:&EllFps)->Result<()> {
	if let Some(path) = &self.mpk {
	    fps.save_mpk(path)?;
	}
	Ok(())
    }
}
