use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::{
    env,
    error::Error,
    fs::{self, File},
    path::Path,
};

static OPENGL_EXTENSIONS: &[&str] = &[];

fn main() -> Result<(), Box<dyn Error>> {
    let out = env::var("OUT_DIR")?;

    // that one giant macro in kernel.rs
    let aa_macro_path = &Path::new(&out).join("_apply_args.macro.rs");
    if !aa_macro_path.is_file() {
        let i_char = |i| char::from(b'a' + i);
        let mut aa_macro = String::with_capacity(16384);
        aa_macro += &"macro_rules! _apply_args {\n    ($args: expr,) => { Ok(()) };\n".to_string();
        for i in 1..16 {
            aa_macro += "    ($args: expr, ";
            for j in 0..i {
                aa_macro += &format!("${}: ident ", i_char(j));
            }
            aa_macro.truncate(aa_macro.len() - 1);
            aa_macro += ") => {{\n        match $args {\n            [";
            for j in 0..i {
                aa_macro.push(i_char(j));
                aa_macro += ", ";
            }
            aa_macro.truncate(aa_macro.len() - 2);
            aa_macro += "] => Ok(( ";
            for j in 0..i {
                aa_macro += &format!("_arg_into!(${0}, {0})?, ", i_char(j));
            }
            aa_macro.truncate(aa_macro.len() - 2);
            aa_macro += " )),\n            _ => unsafe { std::hint::unreachable_unchecked() },\n        }\n    }};\n";
        }
        aa_macro += "}\n";
        fs::write(aa_macro_path, &aa_macro)?;
    }

    // opengl bindings
    let mut bindings = File::create(&Path::new(&out).join("gl_bindings.rs"))?;
    Registry::new(Api::Gl, (3, 3), Profile::Core, Fallbacks::All, &OPENGL_EXTENSIONS)
        .write_bindings(GlobalGenerator, &mut bindings)?;

    Ok(())
}
