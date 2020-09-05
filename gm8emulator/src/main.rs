#![feature(asm, seek_convenience, with_options)]
#![allow(dead_code)] // Shut up.

mod action;
mod asset;
mod game;
mod gml;
mod input;
mod instance;
mod instancelist;
mod math;
mod tile;
mod util;

use std::{
    env, fs,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    process,
};

const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

fn help(argv0: &str, opts: getopts::Options) {
    print!(
        "{}",
        opts.usage(&format!("Usage: {} FILE [options]", match Path::new(argv0).file_name() {
            Some(file) => file.to_str().unwrap_or(argv0),
            None => argv0,
        }))
    );
}

fn main() {
    process::exit(xmain());
}

fn xmain() -> i32 {
    let args: Vec<String> = env::args().collect();
    let process = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "prints this help message");
    opts.optflag("s", "strict", "enable various data integrity checks");
    opts.optflag("t", "singlethread", "parse gamedata synchronously");
    opts.optflag("v", "verbose", "enables verbose logging");
    opts.optflag("r", "realtime", "disables clock spoofing");
    opts.optopt("tmp", "tempdir", "directory to store temporary files in", "DIRECTORY");
    opts.optopt("p", "port", "port to open for external game control (default 15560)", "PORT");
    opts.optopt("n", "project-name", "name of TAS project to create or load", "NAME");
    opts.optopt("f", "replay-file", "path to savestate file to replay", "FILE");
    opts.optmulti("a", "game-arg", "argument to pass to the game", "ARG");

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
        help(&process, opts);
        return EXIT_SUCCESS
    }

    let strict = matches.opt_present("s");
    let multithread = !matches.opt_present("t");
    let spoof_time = !matches.opt_present("r");
    let verbose = matches.opt_present("v");
    let temp_dir = matches.opt_str("tmp").map(|path| {
        let path = PathBuf::from(path);
        if !path.is_dir() {
            if path.exists() {
                panic!("temp directory exists but is not a directory");
            } else {
                std::fs::create_dir_all(&path).unwrap();
            }
        }
        path
    });
    let port = match matches.opt_str("p").map(|x| x.parse::<u16>()).transpose() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("invalid port provided: {}", e);
            return EXIT_FAILURE
        },
    }
    .unwrap_or(15560);
    let project_path = matches.opt_str("n").map(|name| {
        let mut p = env::current_dir().expect("std::env::current_dir() failed");
        p.push("projects");
        p.push(name);
        p
    });
    let replay = matches.opt_str("f").map(|filename| {
        let mut filepath = PathBuf::from(&filename);
        match filepath.extension().and_then(|x| x.to_str()) {
            Some("bin") => {
                let f = fs::File::open(&filepath).unwrap();
                let replay = bincode::deserialize_from::<_, game::SaveState>(BufReader::new(f)).unwrap().into_replay();
                filepath.set_extension("gmtas");
                fs::File::create(&filepath).unwrap().write_all(&bincode::serialize(&replay).unwrap()).unwrap();
                replay
            },

            Some("gmtas") => {
                bincode::deserialize_from::<_, game::Replay>(BufReader::new(fs::File::open(&filepath).unwrap()))
                    .unwrap()
            },

            _ => {
                panic!("Unknown filetype for -f, expected '.bin' or '.gmtas'");
            },
        }
    });
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

    let mut game_args = matches.opt_strs("game-arg");
    game_args.insert(0, input.to_string());
    let game_args = game_args;

    let file_path = Path::new(&input);

    let mut file = match fs::read(file_path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("failed to open '{}': {}", input, err);
            return EXIT_FAILURE
        },
    };

    if verbose {
        println!("loading '{}'...", input);
    }

    #[rustfmt::skip]
    let assets = gm8exe::reader::from_exe(
        &mut file,                              // mut exe: AsRef<[u8]>
        if verbose {                            // logger: Option<Fn(&str)>
            Some(|s: &str| println!("{}", s))
        } else {
            None
        },
        strict,                                 // strict: bool
        multithread,                            // multithread: bool
    );
    let assets = match assets {
        Ok(assets) => assets,
        Err(err) => {
            eprintln!("failed to load '{}' - {}", input, err);
            return EXIT_FAILURE
        },
    };

    let absolute_path = match file_path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to resolve game path: {}", e);
            return EXIT_FAILURE
        },
    };

    let time_nanos = if spoof_time {
        let datetime = chrono::Local::now().naive_local();
        let secs = datetime.timestamp() as u128;
        let nanos = datetime.timestamp_subsec_nanos() as u128;
        Some(secs * 1_000_000_000 + nanos)
    } else {
        None
    };

    let encoding = encoding_rs::SHIFT_JIS; // TODO: argument

    let mut components = match game::Game::launch(assets, absolute_path, time_nanos, game_args, temp_dir, encoding) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to launch game: {}", e);
            return EXIT_FAILURE
        },
    };

    if let Err(err) = if let Some(path) = project_path {
        components.record(path, port)
    } else {
        if let Some(replay) = replay { components.replay(replay) } else { components.run() }
    } {
        println!("Runtime error: {}", err);
        EXIT_FAILURE
    } else {
        EXIT_SUCCESS
    }
}
