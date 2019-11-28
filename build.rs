use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::{env, error::Error, fs::File, path::Path};

static OPENGL_EXTENSIONS: &[&str] = &[];

fn main() -> Result<(), Box<dyn Error>> {
    let out = env::var("OUT_DIR")?;
    let mut bindings = File::create(&Path::new(&out).join("gl_bindings.rs"))?;
    Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::All, &OPENGL_EXTENSIONS)
        .write_bindings(GlobalGenerator, &mut bindings)?;
    Ok(())
}
