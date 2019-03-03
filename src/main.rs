#![feature(asm)] // Usage: xmath
#![feature(try_trait)]

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
        println!("Usage: gm8emu <exe_path> [--verbose]");
        return Ok(());
    }

    let mut verbose = false;
    let mut path: Option<&String> = None;
    for arg in args.iter() {
        match arg.as_ref() {
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
        let game = Game::from_exe(data, verbose);

        match game {
            Ok(_) => println!("Parsing OK!"),
            Err(err) => println!("{}", err),
        }
    }

    Ok(())
}
