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
mod types;
mod util;

use game::{
    savestate::{self, SaveState},
    Game, PlayType, Replay,
};
use std::{
    env, fs,
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
    opts.optopt("l", "no-framelimit-until", "disables the frame-limiter until specified frame", "FRAME");
    opts.optopt("n", "project-name", "name of TAS project to create or load", "NAME");
    opts.optopt("f", "replay-file", "path to savestate file to replay", "FILE");
    opts.optopt("o", "output-file", "output savestate name in replay mode", "FILE.bin");
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
    let frame_limit_at = matches.opt_str("l").map(|frame| {
        match frame.parse::<usize>() 
        {
            Ok(f) => f,
            Err(e) => {
                panic!("{}", e);
            },
        }
    }).unwrap_or(0);
    let frame_limiter = !matches.opt_present("l");
    let verbose = matches.opt_present("v");
    let output_bin = matches.opt_str("o").map(PathBuf::from);
    let project_path = matches.opt_str("n").map(|name| {
        let mut p = env::current_dir().expect("std::env::current_dir() failed");
        p.push("projects");
        p.push(name);
        p
    });

    if let Some(bin) = &output_bin {
        if bin.extension().and_then(|x| x.to_str()) != Some("bin") {
            eprintln!("invalid output file for -o: must be a .gmtas file");
            return EXIT_FAILURE
        }
    }

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
    let replay = match matches
        .opt_str("f")
        .map(|filename| {
            let filepath = PathBuf::from(&filename);
            match filepath.extension().and_then(|x| x.to_str()) {
                Some("bin") => match SaveState::from_file(&filepath, &mut savestate::Buffer::new()) {
                    Ok(state) => Ok(state.into_replay()),
                    Err(e) => Err(format!("couldn't load {:?}: {:?}", filepath, e)),
                },

                Some("gmtas") => match Replay::from_file(&filepath) {
                    Ok(replay) => Ok(replay),
                    Err(e) => Err(format!("couldn't load {:?}: {:?}", filepath, e)),
                },

                _ => Err("unknown filetype for -f, expected '.bin' or '.gmtas'".into()),
            }
        })
        .transpose()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return EXIT_FAILURE
        },
    };

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
        PlayType::Record
    } else if replay.is_some() {
        PlayType::Replay
    } else {
        PlayType::Normal
    };

    let mut components =
        match Game::launch(assets, absolute_path, game_args, temp_dir, encoding, frame_limiter, frame_limit_at, play_type) {
            Ok(g) => g,
            Err(e) => {
                eprintln!("Failed to launch game: {}", e);
                return EXIT_FAILURE
            },
        };

    let time_now = gml::datetime::now_as_nanos();

    if let Err(err) = if let Some(path) = project_path {
        components.spoofed_time_nanos = Some(time_now);
        components.record(path);
        Ok(())
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
            components.replay(replay, output_bin)
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
