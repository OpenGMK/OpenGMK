#![allow(dead_code)]

mod panel;

use shared::message::{Information, Message, MessageStream};
use std::{env, path::Path, process};

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

    let bind_addr = format!("127.0.0.1:15560");
    println!("Waiting on TCP connection to {}", bind_addr);
    let listener = std::net::TcpListener::bind(bind_addr).unwrap();

    let mut emu = process::Command::new("gm8emulator.exe");
    let _emu_handle = if verbose { emu.arg(input).arg("v") } else { emu.arg(input) }
        .arg("-n")
        .arg(project_name)
        .arg("-p")
        .arg("15560")
        .spawn()
        .expect("failed to execute process");

    let (mut stream, remote_addr) = listener.accept().unwrap();
    stream.set_nonblocking(true).unwrap();
    println!("Connection established with {}", &remote_addr);

    let keys = panel.key_buttons.iter().map(|x| x.key).collect::<Vec<_>>();
    let buttons = Vec::new();
    println!("Sending 'Hello' with {} keys, {} mouse buttons", keys.len(), buttons.len());
    stream.send_message(&Message::Hello { keys_requested: keys, mouse_buttons_requested: buttons }).unwrap();

    loop {
        match stream.receive_message::<Information>(&mut panel.read_buffer) {
            Ok(None) => return EXIT_SUCCESS,
            Ok(Some(Some(s))) => match s {
                Information::Update { keys_held, mouse_buttons_held: _, mouse_location: _, frame_count: _, seed: _, instance: _ } => {
                    println!("Received update from game client");
                    for key in panel.key_buttons.iter_mut() {
                        if keys_held.contains(&key.key) {
                            key.state = panel::KeyButtonState::Held;
                        } else {
                            key.state = panel::KeyButtonState::Neutral;
                        }
                    }
                    break
                },
                m => {
                    eprintln!("Unexpected response to 'Hello': {:?}", m);
                    return EXIT_FAILURE
                },
            },
            Ok(Some(None)) => (),
            Err(e) => {
                eprintln!("error reading from tcp stream: {}", e);
                return EXIT_FAILURE
            },
        }
    }

    loop {
        match stream.receive_message::<Information>(&mut panel.read_buffer) {
            Ok(None) => break,
            Ok(Some(Some(s))) => println!("Got TCP message: '{:?}'", s),
            Ok(Some(None)) => (),
            Err(e) => {
                eprintln!("error reading from tcp stream: {}", e);
                return EXIT_FAILURE
            },
        }

        panel.update(&mut stream);
        panel.draw();
        if panel.window.close_requested() {
            break
        }
    }

    EXIT_SUCCESS
}
