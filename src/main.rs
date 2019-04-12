mod assets;
mod bytes;
mod game;
mod types;
mod util;
mod xmath;

use crate::game::Game;
use std::env;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() < 1 {
        print!("{}", include_str!("../incl/default"));
        return Ok(());
    }

    let print_usage = || print!("{}", include_str!("../incl/usage"));

    let mut verbose = false;
    let mut strict = true;
    let mut path: Option<&String> = None;
    let mut dll_dump: Option<&String> = None;

    let mut args_it = args.iter();
    while let Some(arg) = args_it.next() {
        match arg.as_ref() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }

            "-D" | "--dump-dll" => {
                if let Some(path) = args_it.next() {
                    dll_dump = Some(path);
                } else {
                    println!("Invalid usage of dump-dll, out-path not provided.");
                    print_usage();
                    std::process::exit(1);
                }
            }

            "-l" | "--lazy" => strict = false,

            "--verbose" => verbose = true,

            _ => {
                if let Some(path) = &path {
                    println!("Can't open multiple games at once! ({} and {})", path, arg);
                    std::process::exit(1);
                } else {
                    path = Some(arg);
                }
            }
        }
    }

    if let Some(path) = path {
        let data = fs::read(path)?;
        let game = Game::from_exe(data, strict, verbose, dll_dump);

        match game {
            Ok(_) => println!("Parsing OK!"),
            Err(err) => println!("{}", err),
        }
    }

    Ok(())
}
