use super::*;

pub const CMD : Subcommand = Subcommand {
    name:"list-recs",
    synopsis:"Lists the records of a NAT file",
    run,
    help:"\
    Mandatory arguments:
       --input  IN.nat
		Path of input IASI NAT file (L1C or L2)"
};

fn run(mut args:Arguments)->Result<()> {
    let input_path : OsString = args.value_from_str("--input")?;

    let rest = args.finish();
    if !rest.is_empty() {
	bail!("Unhandled arguments: {:?}; try --help",rest);
    }

    info!("Opening NAT file {:?}",input_path);
    let fd_in = File::open(&input_path)?;
    let mut br = BufReader::new(fd_in);
    let recs = Grh::read_recs(&mut br)?;
    println!("{:4} {:17} {:3} {:26} {:26} {:8} {:16}",
	     "Rec#","Kind","Ins","Start","End","Position","Size");
    for (irec,rec) in recs.iter().enumerate() {
	let Grh { record_kind,
		  instrument_group,
		  record_size,
		  record_start_time,
		  record_end_time,
		  record_pos } = *rec;
	let t1 = record_start_time.to_gregorian_hms()?;
	let t2 = record_end_time.to_gregorian_hms()?;
	println!("{:4} {:17} {:3} {:26} {:26} {:8} {:16}",
		 irec,
		 format!("{}",record_kind),
		 instrument_group,
		 t1,
		 t2,
		 record_pos,
		 record_size,
	);
    }
    Ok(())
}
