use super::*;

pub const CMD : Subcommand = Subcommand {
    name:"show-mphr",
    synopsis:"Show the MPHR of a NAT file",
    run,
    help:Seq::One(&"\
Mandatory arguments
===================
--input	IN.nat
	Path of input IASI NAT file (L1C or L2)")
};

fn run(mut args:Arguments)->Result<()> {
    let input_path : OsString = args.value_from_str("--input")?;
    finish_args(args)?;

    let fd = File::open(input_path)?;
    let mut br = BufReader::new(fd);
    let recs = Grh::read_recs(&mut br)?;
    for rec in &recs {
        let kind = &rec.record_kind;
        match kind {
            GrhRecordKind::Mphr => {
                let mphr = Mphr::read_bin(&mut br,rec)?;
                println!("{:#?}",mphr);
                break;
            },
            _ => ()
        }
    }
    Ok(())
}
