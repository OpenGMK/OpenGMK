use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};

use std::{env, error::Error, fs::File, path::Path};

const OPENGL_VER_MAJOR: u8 = 3;
const OPENGL_VER_MINOR: u8 = 3;
const OPENGL_PROFILE: Profile = Profile::Core;
static OPENGL_EXTENSIONS: &[&str] = &[];

fn main() -> Result<(), Box<dyn Error>> {
    let out = env::var("OUT_DIR")?;
    let mut bindings = File::create(&Path::new(&out).join("gl_bindings.rs"))?;
    Registry::new(
        Api::Gl,
        (OPENGL_VER_MAJOR, OPENGL_VER_MINOR),
        OPENGL_PROFILE,
        Fallbacks::All,
        &OPENGL_EXTENSIONS,
    )
    .write_bindings(GlobalGenerator, &mut bindings)?;
    Ok(())
}
