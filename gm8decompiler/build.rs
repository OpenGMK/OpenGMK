use std::{env, error::Error, process::Command};
use time::OffsetDateTime;

fn main() -> Result<(), Box<dyn Error>> {
    // build date
    let time = OffsetDateTime::now_utc();
    println!("cargo:rustc-env=BUILD_DATE={}/{}/{}", time.year(), time.month() as u8, time.day());

    // git hash
    let output = Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output()?;
    let git_hash = String::from_utf8(output.stdout)?;
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // target triple
    let target_triple = env::var("TARGET")?;
    println!("cargo:rustc-env=TARGET_TRIPLE={}", target_triple);

    // icon
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/logo/gm8dec.ico");
        res.compile()?;
    }

    Ok(())
}
