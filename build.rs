use std::{error::Error, process::Command};
use chrono::{Datelike, Local};

fn main() -> Result<(), Box<dyn Error>> {
    // build date
    let time = Local::now();
    println!("cargo:rustc-env=BUILD_DATE={}/{}/{}", time.year(), time.month(), time.day());

    // git hash
    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output()?;
    let git_hash = String::from_utf8(output.stdout)?;
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    Ok(())
}
