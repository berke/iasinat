use std::process::Command;
use anyhow::Result;

fn main()->Result<()> {
    let cmd_res = Command::new("git")
	.args(["describe","--always"])
	.output()?;
    let descr = String::from_utf8_lossy(&cmd_res.stdout);

    let cmd_res = Command::new("date")
	.args(["--iso-8601"])
	.output()?;
    let stamp = String::from_utf8_lossy(&cmd_res.stdout);

    println!("cargo::rustc-env=IASINAT_COMMIT={}",descr);
    println!("cargo::rustc-env=IASINAT_BUILD_TIMESTAMP={}",stamp);
    Ok(())
}
