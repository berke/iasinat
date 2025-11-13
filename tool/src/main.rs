mod cmp1c_netcdf;
mod cmp2_netcdf;
mod list_recs;
mod nat1c_to_netcdf;
mod nat2_to_netcdf;
mod netcdf_cmp;

use std::{
    fmt::{
	Display,
	Formatter,
	Write
    },
    fs::File,
    ffi::OsString,
    io::{
	BufReader,
    }
};

use anyhow::{
    bail,
    Result
};

use ndarray::{
    Array2,
    Array3,
    Array4,
    ArrayD,
    Dimension,
    s
};

use log::{
    trace,
    info,
    error,
    warn
};

use pico_args::Arguments;

use iasinat_lib::prelude::*;
use netcdf as nc;

use nc::{
    AttributeValue,
    Variable
};

#[cfg(feature="footprints")]
use circfp::{
    Geodetic360,
    EllipsoidConverter,
    FpEstimator,
    ObservationEstimator,
    WGS84
};

pub struct Subcommand {
    pub name:&'static str,
    pub synopsis:&'static str,
    pub help:fn()->&'static [&'static str],
    pub run:fn(Arguments)->Result<()>
}

const SUBCMDS : &[&Subcommand] = &[
    &cmp1c_netcdf::CMD,
    &cmp2_netcdf::CMD,
    &list_recs::CMD,
    &nat1c_to_netcdf::CMD,
    &nat2_to_netcdf::CMD,
];

const PROGRAM_NAME : &str = "iasinat by ExH R&D S.A.R.L. <bd@exhrd.fr>";

pub fn finish_args(args:Arguments)->Result<()> {
    let rest = args.finish();
    if !rest.is_empty() {
	bail!("Unhandled arguments: {:?}; try --help",rest);
    }
    Ok(())
}

fn do_version(_args:Arguments)->Result<()> {
    println!("{}",PROGRAM_NAME);
    println!("  Commit: {}",env!("IASINAT_COMMIT"));
    println!("  Build date: {}",env!("IASINAT_BUILD_TIMESTAMP"));
    Ok(())
}

fn do_help(_args:Arguments,cmd:Option<&Subcommand>)->Result<()> {
    let progname = std::env::args().next().unwrap();
    match cmd {
	None => {
	    println!("Usage: {} SUBCOMMAND ARGUMENTS",progname);
	    println!();
	    println!("Subcommands:");
	    for Subcommand { name,synopsis,.. } in SUBCMDS {
		println!("  {:20} {}",name,synopsis);
	    }
	    println!();
	    println!("Run {} SUBCOMMAND --help for details",progname);
	},
	Some(Subcommand { name,synopsis,help,.. }) => {
	    println!("{}: {}",name,synopsis);
	    println!();
	    println!("Usage: {} {} ARGUMENTS",progname,name);
	    println!();
	    for h in help() {
		println!("{}",h);
	    }
	}
    }
    Ok(())
}

fn main()->Result<()> {
    let mut args = Arguments::from_env();

    if args.contains("--version") {
	do_version(args)
    } else {
	let verbose = args.contains("--verbose");
	let trace = args.contains("--trace");
	let debug = args.contains("--debug");
	let help = args.contains("-h") || args.contains("--help");
	
	simple_logger::SimpleLogger::new()
	    .with_level(
		if trace { log::LevelFilter::Trace }
		else if debug { log::LevelFilter::Debug }
		else if verbose { log::LevelFilter::Info }
		else { log::LevelFilter::Warn })
	    .init()?;
	match args.subcommand()?.as_deref() {
	    Some(sc) => {
		for cmd @ Subcommand { name,run,.. } in SUBCMDS {
		    if sc == *name {
			if help {
			    return do_help(args,Some(cmd));
			} else {
			    return run(args);
			}
		    }
		}
		bail!("Unknown subcommand {}; try --help",sc);
	    }
	    None => {
		if help {
		    do_help(args,None)
		} else {
		    bail!("No subcommand specified; try --help")
		}
	    }
	}
    }
}

fn add_metadata(fd_out:&mut nc::FileMut,mphr:&Mphr,subcommand:&str)->Result<()> {
    trace!("Adding metadata");
    let _attr = fd_out.add_attribute("product_name",mphr.product_name.clone())?;
    let _conv_name = fd_out.add_attribute("converter_name",
					  PROGRAM_NAME.to_string());
    let _conv_subcommand = fd_out.add_attribute("converter_subcommand",
						subcommand.to_string());
    let _conv_commit = fd_out.add_attribute(
	"converter_commit",
	env!("IASINAT_COMMIT").to_string());
    let _conv_stamp = fd_out.add_attribute(
	"converter_build_timestamp",
	env!("IASINAT_BUILD_TIMESTAMP").to_string());
    Ok(())
}
