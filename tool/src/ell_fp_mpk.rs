use super::*;

use footprint::{
    amcut,
    Footprint,
    Footprints,
    geo::{
	algorithm::contains::Contains,
	Point
    },
    poly_utils
};

pub struct EllFpMpkOptions {
    path:Option<OsString>,
    amcut:bool,
    check:bool
}

pub const HELP : Seq<'static,&'static str> = Seq::One(&"\
--mpk-output PATH
             Save footprints in MPK footprint format
--mpk-amcut  Cut footprints that cross the antimeridian line
--mpk-check  Make sure that the center point is contained in
             the multipolygon.  (May fail without --mpk-amcut.)
");

impl EllFpMpkOptions {
    pub fn from_args(args:&mut Arguments)->Result<Self> {
	let path : Option<OsString> = args.opt_value_from_str("--mpk-output")?;
	let amcut = args.contains("--mpk-amcut");
	let check = args.contains("--mpk-check");
	Ok(Self { path,amcut,check })
    }

    pub fn active(&self)->bool {
	self.path.is_some()
    }
}

pub trait EllFpMpk {
    fn save_mpk(&self,mphr:&Mphr,opts:&EllFpMpkOptions)->Result<()>;
}

impl EllFpMpk for EllFps {
    fn save_mpk(&self,mphr:&Mphr,opts:&EllFpMpkOptions)->Result<()> {
	if let &EllFpMpkOptions { path:Some(ref path),amcut,check } = opts {
	    let Self { coords,lats,lons,.. } = self;
	    let (nline,_,_,npoint) = lats.dim();

	    let mut footprints = Vec::with_capacity(nline);
	    let mut ncross = 0;

	    let dataset_id = &mphr.product_name;
	    let orbit = mphr.orbit_start as usize;
	    let platform = &mphr.spacecraft_id;
	    let instrument = "IASI";

	    for iline in 0..nline {
		for j in 0..SNOT {
		    for i in 0..PN {
			let mut ring = Vec::new();
			for k in 0..npoint {
			    ring.push(
				(lons[[iline,j,i,k]] as f64,
				 lats[[iline,j,i,k]] as f64));
			}
			let mut outline = Vec::new();

			if amcut {
			    if amcut::cut_and_push(&mut outline,ring) {
				ncross += 1;
			    }
			} else {
			    outline.push(vec![ring]);
			}

			if check {
			    let mp = poly_utils::outline_to_multipolygon(
				&outline);
			    let x = coords[[iline,j,i,IFPCOORD_LON]];
			    let y = coords[[iline,j,i,IFPCOORD_LAT]];
			    if !mp.contains(&Point::new(x,y)) {
				warn!("Granule {} scan {} pixel {} \
				       coordinates x={} y={} \
				       not contained in\n{:#?}",
				      iline,j + 1,i + 1,x,y,mp);
			    }
			}

			let id = format!("{}/{}/{}/{}",
					 dataset_id,
					 iline,
					 j + 1,
					 i + 1);

			let fp = Footprint{
			    orbit,
			    id:id.to_string(),
			    platform:platform.to_string(),
			    instrument:instrument.to_string(),
			    time_interval:(0.0,0.0), // xxx
			    outline
			};
			footprints.push(fp);

		    }
		}
	    }

	    if amcut {
		info!("Number of antimeridian crossings: {}",ncross);
	    }

	    let fps = Footprints{ footprints };
	    fps.save_to_file(path)?;
	}
	Ok(())
    }
}
