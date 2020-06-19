#![allow(dead_code)]

#[path = "../../atlas.rs"]
pub mod atlas;

#[path = "../../input.rs"]
pub mod input;

#[path = "../../game/tas/message.rs"]
pub mod message;

#[path = "../../types.rs"]
pub mod types;

mod panel;
mod render;
mod window;

use std::{
    env,
    net::{SocketAddr, TcpStream},
    path::Path,
    process,
};
use message::MessageStream;

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

const WINDOW_WIDTH: u32 = 300;
const WINDOW_HEIGHT: u32 = 750;

fn main() {
    process::exit(xmain());
}

fn xmain() -> i32 {
    let args: Vec<String> = env::args().collect();
    let process_name = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "prints this help message");
    opts.optopt("n", "project-name", "name of TAS project to create or load", "NAME");
    opts.optflag("v", "verbose", "enables verbose logging");

    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(fail) => {
            use getopts::Fail::*;
            match fail {
                ArgumentMissing(arg) => eprintln!("missing argument {}", arg),
                UnrecognizedOption(opt) => eprintln!("unrecognized option {}", opt),
                OptionMissing(opt) => eprintln!("missing option {}", opt),
                OptionDuplicated(opt) => eprintln!("duplicated option {}", opt),
                UnexpectedArgument(arg) => eprintln!("unexpected argument {}", arg),
            }
            return EXIT_FAILURE
        },
    };

    if args.len() < 2 || matches.opt_present("h") {
        print!(
            "{}",
            opts.usage(&format!("Usage: {} FILE -n PROJECT-NAME [-v]", match Path::new(&process_name).file_name() {
                Some(file) => file.to_str().unwrap_or(&process_name),
                None => &process_name,
            }))
        );
        return EXIT_SUCCESS
    }

    let verbose = matches.opt_present("v");
    let input = {
        if matches.free.len() == 1 {
            &matches.free[0]
        } else if matches.free.len() > 1 {
            eprintln!("unexpected second input {}", matches.free[1]);
            return EXIT_FAILURE
        } else {
            eprintln!("no input file");
            return EXIT_FAILURE
        }
    };
    let project_name = match matches.opt_str("n") {
        Some(p) => p,
        None => {
            eprintln!("missing required argument: -n project-name");
            return EXIT_FAILURE
        },
    };

    println!("input {}, project name {}, verbose {}", input, project_name, verbose);

    let mut panel = match panel::ControlPanel::new() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error starting control panel: {}", e);
            return EXIT_FAILURE
        },
    };

    let mut emu = process::Command::new("gm8emulator.exe");
    let _emu_handle = if verbose { emu.arg(input).arg("v") } else { emu.arg(input) }
        .arg("-n")
        .arg(project_name)
        .arg("-p")
        .arg("15560")
        .spawn()
        .expect("failed to execute process");

    let mut stream = match TcpStream::connect(&SocketAddr::from(([127, 0, 0, 1], 15560))) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("could not open outbound TCP connection: {}", e);
            return EXIT_FAILURE
        },
    };

    stream.send_message(String::from("Hello World")).expect("Couldn't send message");

    loop {
        panel.draw();
        panel.window.process_events();
        if panel.window.close_requested() {
            break;
        }
    }

    EXIT_SUCCESS
}
