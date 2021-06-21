#![feature(bindings_after_at, seek_stream_len)]

mod action;
mod asset;
mod game;
mod gml;
mod handleman;
mod imgui;
mod input;
mod instance;
mod instancelist;
mod math;
mod render;
mod tile;
mod util;
mod types;

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
    opts.optflag("l", "no-framelimit", "disables the frame-limiter");
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
    let frame_limiter = !matches.opt_present("l");
    let verbose = matches.opt_present("v");
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
    let temp_dir = project_path.as_ref().map(|proj_path| {
        // attempt to find temp dir in project path
        std::fs::read_dir(proj_path)
            .ok()
            .and_then(|iter| {
                iter.filter_map(|x| x.ok())
                    .find(|p| {
                        p.metadata().ok().filter(|p| p.is_dir()).is_some()
                            && p.file_name().to_str().filter(|s| s.starts_with("gm_ttt_")).is_some()
                    })
                    .map(|entry| entry.path())
            })
            // if we can't find one, make one
            .unwrap_or_else(|| {
                let mut random_int = [0u8; 4];
                getrandom::getrandom(&mut random_int).expect("Couldn't generate a random number");
                let path = [proj_path.clone(), format!("gm_ttt_{}", u32::from_le_bytes(random_int) % 100000).into()]
                    .iter()
                    .collect();
                if let Err(e) = std::fs::create_dir_all(&path) {
                    println!("Could not create temp folder: {}", e);
                    println!("If this game uses the temp folder, it will most likely crash.");
                }
                path
            })
    });
    let can_clear_temp_dir = temp_dir.is_none();
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

    let encoding = encoding_rs::SHIFT_JIS; // TODO: argument

    let play_type = if project_path.is_some() {
        game::PlayType::Record
    } else if replay.is_some() {
        game::PlayType::Replay
    } else {
        game::PlayType::Normal
    };

    let mut components = match game::Game::launch(assets, absolute_path, game_args, temp_dir, encoding, frame_limiter, play_type) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to launch game: {}", e);
            return EXIT_FAILURE
        },
    };

    let time_now = gml::datetime::now_as_nanos();

    if let Err(err) = if let Some(path) = project_path {
        components.spoofed_time_nanos = Some(time_now);
        components.record(path, port)
    } else {
        // cache temp_dir and included files because the other functions take ownership
        let temp_dir: Option<PathBuf> = if can_clear_temp_dir {
            Some(components.decode_str(components.temp_directory.as_ref()).into_owned().into())
        } else {
            None
        };
        let files_to_delete = components
            .included_files
            .iter()
            .filter(|i| i.remove_at_end)
            .map(|i| PathBuf::from(components.decode_str(i.name.as_ref()).into_owned()))
            .collect::<Vec<_>>();
        let result = if let Some(replay) = replay {
            components.replay(replay)
        } else {
            components.spoofed_time_nanos = if spoof_time { Some(time_now) } else { None };
            components.run()
        };
        for file in files_to_delete.into_iter() {
            std::fs::remove_file(file).ok();
        }
        if let Some(temp_dir) = temp_dir {
            std::fs::remove_dir_all(temp_dir).ok();
        }
        result
    } {
        println!("Runtime error: {}", err);
        EXIT_FAILURE
    } else {
        EXIT_SUCCESS
    }
}
