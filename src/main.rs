#![allow(dead_code)] // Shut up.

mod bytes;
mod game;
mod gml;
mod types;
mod util;
mod xmath;

use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() < 1 {
        print!("{}", include_str!("incl/default"));
        return Ok(());
    }

    let print_usage = || print!("{}", include_str!("incl/usage"));

    let mut path: Option<&String> = None;
    let mut dump_dll_path: Option<&Path> = None;
    let mut strict = true;
    let mut verbose = false;

    let mut argi = args.iter();
    while let Some(arg) = argi.next() {
        match arg.as_ref() {
            "-h" | "--help" => {
                print_usage();
                return Ok(());
            }

            "-D" | "--dump-dll" => {
                if let Some(path) = argi.next() {
                    dump_dll_path = Some(Path::new(path));
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

    fn l_print(s: &str) {
        println!("{}", s);
    }

    let assets = if let Some(path) = path {
        let data = fs::read(path)?;
        let assets = gm8x::reader::from_exe(data, strict, if verbose { Some(l_print) } else { None }, dump_dll_path);

        match assets {
            Ok(a) => {
                println!("Parsing OK!");
                a
            }
            Err(err) => {
                println!("{}", err);
                std::process::exit(1);
            }
        }
    } else {
        println!("No path wtf");
        std::process::exit(1);
    };

    // Start window, for now, I guess
    let icon = assets.icon_data.and_then(|data| game::icon_from_win32(&data));

    let (event_loop, window) = game::window("k3", 800, 608, icon, &assets.settings).unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        winit::event::Event::WindowEvent {
            event: winit::event::WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => *control_flow = winit::event_loop::ControlFlow::Exit,
        _ => *control_flow = winit::event_loop::ControlFlow::Wait,
    });
}
