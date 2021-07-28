use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};
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
        for i in 1..=16 {
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
        .write_bindings(StructGenerator, &mut bindings)?;

    // WGL (Windows OpenGL) Bindings
    if cfg!(target_os = "windows") {
        let mut file = File::create(&Path::new(&out).join("wgl_bindings.rs"))?;
        Registry::new(Api::Wgl, (1, 0), Profile::Core, Fallbacks::All, [
            "WGL_ARB_create_context",
            "WGL_ARB_create_context_profile",
            "WGL_ARB_extensions_string",
            "WGL_EXT_swap_control",
        ])
        .write_bindings(StructGenerator, &mut file)?;
    }

    // GLX (OpenGL Extension for X) Bindings
    if cfg!(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
    )) {
        let mut file = File::create(&Path::new(&out).join("glx_bindings.rs"))?;
        Registry::new(Api::Glx, (1, 4), Profile::Core, Fallbacks::All, [
            "GLX_ARB_create_context",
            "GLX_ARB_create_context_profile",
            "GLX_EXT_swap_control",
        ])
        .write_bindings(StructGenerator, &mut file)?;
    }

    // Windows-specific resources
    #[cfg(target_os = "windows")]
    {
        // This code reduces size of the manifest resource. Some important notes about:
        // * Last char trickery is necessary to prevent gluing attributes.
        // * Windows XML parser doesn't understand LF line endings in manifests.
        // * Indentation must use tabs to avoid removing meaningful spaces from values.
        //   They're also more compact than multiple spaces.
        // * To prevent excessive spaces before '/>' in single (self-closing) tags,
        //   they must reside on the same line with the last attribute.
        let manifest = {
            let mut last = '\0';
            fs::read_to_string("data/gm8emulator.exe.manifest")?
                .chars()
                .filter_map(|x| match x {
                    '\r' | '\t' => None,
                    '\n' => {
                        if last != '>' {
                            Some(' ')
                        } else {
                            None
                        }
                    },
                    _ => {
                        last = x;
                        Some(x)
                    },
                })
                .collect::<String>()
        };

        winres::WindowsResource::new().set_icon("../assets/logo/opengmk.ico").set_manifest(&manifest).compile()?;
    }

    Ok(())
}
