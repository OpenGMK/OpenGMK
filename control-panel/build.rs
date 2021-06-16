extern crate winres;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // icon
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/logo/tas.ico");
        res.compile()?;
    }

    Ok(())
}
