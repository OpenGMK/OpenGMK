#![allow(dead_code)] // Shut up.

mod assets;
mod bytes;
mod game;
mod gml;
mod types;
mod util;
mod xmath;

use crate::game::{parser::ParserOptions, Game};
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() < 1 {
        print!("{}", include_str!("../incl/default"));
        return Ok(());
    }

    let print_usage = || print!("{}", include_str!("../incl/usage"));
    let mut options = ParserOptions::new();
    let mut path: Option<&String> = None;
    let mut argi = args.iter();

    while let Some(arg) = argi.next() {
        match arg.as_ref() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }

            "-D" | "--dump-dll" => {
                if let Some(path) = argi.next() {
                    options.dump_dll = Some(Path::new(path));
                } else {
                    println!("Invalid usage of dump-dll, out-path not provided.");
                    print_usage();
                    std::process::exit(1);
                }
            }

            "-l" | "--lazy" => options.strict = false,

            "--verbose" => options.log = true,

            "--test-sdl2" => {
                let sdl = sdl2::init().unwrap();
                let video_subsystem = sdl.video().unwrap();
                let _window = video_subsystem
                    .window("gm8emu ✨", 800, 600)
                    .build()
                    .unwrap();

                let mut event_pump = sdl.event_pump().unwrap();
                'main: loop {
                    for event in event_pump.poll_iter() {
                        match event {
                            sdl2::event::Event::Quit { .. } => break 'main,
                            _ => {}
                        }
                    }
                }

                return Ok(());
            }

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
        let game = Game::from_exe(data, &options);

        match game {
            Ok(_) => println!("Parsing OK!"),
            Err(err) => println!("{}", err),
        }
    }

    Ok(())
}
