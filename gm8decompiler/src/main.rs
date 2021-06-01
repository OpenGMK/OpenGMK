use gm8exe::GameVersion;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

pub mod collision;
pub mod deobfuscate;
pub mod gmk;
pub mod mappings;
pub mod zlib;

static INFO_STRING: &str = concat!(
    "GM8Decompiler v",
    env!("CARGO_PKG_VERSION"),
    " for ",
    env!("TARGET_TRIPLE"),
    " - built on ",
    env!("BUILD_DATE"),
    ", #",
    env!("GIT_HASH"),
);

// Know to "press any key" but only if double-clicked in WinExplorer or whatever.
#[cfg(windows)]
fn is_cmd(argv_0: &str) -> bool {
    let is_argv0_absolute = Path::new(argv_0).is_absolute();
    let is_msys2 = env::var("MSYSTEM").is_ok();

    is_argv0_absolute && !is_msys2
}
#[cfg(windows)]
fn pause(tip: bool) {
    extern "C" {
        fn _getch() -> std::os::raw::c_int;
    }
    if tip {
        println!("\nTip: To decompile a game, click and drag it on top of the executable.");
    }
    println!("<< Press Any Key >>");
    let _ = unsafe { _getch() };
}
#[cfg(not(windows))]
fn is_cmd(_argv_0: &str) -> bool {
    false
}
#[cfg(not(windows))]
fn pause(_tip: bool) {}

fn main() {
    println!("{}", INFO_STRING);

    let args: Vec<String> = env::args().collect();
    assert!(!args.is_empty());
    let process_path = args[0].as_str();
    let should_pause = is_cmd(process_path);

    // set up getopts to parse our command line args
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help message")
        .optflag("l", "lazy", "disable various data integrity checks")
        .optflag("v", "verbose", "enable verbose logging for decompilation")
        .optopt("d", "deobfuscate", "set deobfuscation mode auto/on/off (default=auto)", "")
        .optflag("p", "preserve", "preserve broken events (instead of trying to fix them)")
        .optflag("s", "singlethread", "decompile gamedata synchronously (lower RAM usage)")
        .optopt("o", "output", "specify output filename", "FILE");

    // parse command line arguments
    let matches = match opts.parse(&args[1..]) {
        Ok(matches) => matches,
        Err(err) => {
            use getopts::Fail::*;
            match err {
                ArgumentMissing(arg) => eprintln!("Missing argument: {}", arg),
                UnrecognizedOption(opt) => eprintln!("Unrecognized option: {}", opt),
                OptionMissing(opt) => eprintln!("Missing option: {}", opt),
                OptionDuplicated(opt) => eprintln!("Duplicated option: {}", opt),
                UnexpectedArgument(arg) => eprintln!("Unexpected argument: {}", arg),
            }
            if should_pause {
                pause(true);
            }
            process::exit(1);
        },
    };

    // print help message if requested OR no input files
    if matches.opt_present("h") || matches.free.is_empty() {
        // If the getopts2 usage generator didn't suck this much,
        // I wouldn't have to resort to this.
        // TODO: Get a better argument parser in general.
        println!(
            "Usage: {} FILENAME [options]

Options:
    -h, --help                print this help message
    -l, --lazy                disable various data integrity checks
    -v, --verbose             enable verbose logging for decompilation
    -d, --deobfuscate <mode>  set deobfuscation mode auto/on/off (defaults to auto)
    -p, --preserve            preserve broken events (instead of trying to fix them)
    -s, --singlethread        decompile gamedata synchronously (lower RAM usage)
    -o, --output <file>       specify output filename",
            process_path
        );
        if should_pause {
            pause(true);
        }
        process::exit(0); // once the user RTFM they can run it again
    }

    // print error message if multiple inputs were provided
    if matches.free.len() > 1 {
        eprintln!(
            concat!("Unexpected input: {}\n", "Tip: Only one input gamefile is expected at a time!",),
            matches.free[1]
        );
        if should_pause {
            pause(true);
        }
        process::exit(1);
    }

    // extract flags & input path
    let input = &matches.free[0];
    let lazy = matches.opt_present("l");
    let singlethread = matches.opt_present("s");
    let verbose = matches.opt_present("v");
    let deobfuscate = match matches.opt_str("d").as_deref() {
        Some("on") => deobfuscate::Mode::On,
        Some("off") => deobfuscate::Mode::Off,
        Some("auto") | None => deobfuscate::Mode::Auto,
        Some(x) => {
            eprintln!("Invalid deobfuscator setting: {} (valid settings are on/off/auto)", x);
            process::exit(1);
        },
    };
    let out_path = matches.opt_str("o");
    let preserve = matches.opt_present("p");
    // no_pause extracted before help

    // print flags for confirmation
    println!("Input file: {}", input);
    if lazy {
        println!("Lazy mode ON: data integrity checking disabled");
    }
    if verbose {
        println!("Verbose logging ON: verbose console output enabled");
    }
    match deobfuscate {
        deobfuscate::Mode::On => println!("Deobfuscation ON: will standardise GML code"),
        deobfuscate::Mode::Off => println!("Deobfuscation OFF: will ignore obfuscation"),
        _ => (),
    }
    if singlethread {
        println!("Single-threaded mode ON: process will not start new threads (slow)");
    }
    if let Some(path) = &out_path {
        println!("Specified output path: {}", path);
    }
    if preserve {
        println!("Preserve mode ON: broken events will be preserved and will not be fixed");
    }

    // resolve input path
    let input_path = Path::new(input);
    if !input_path.is_file() {
        eprintln!("Input file '{}' does not exist.", input);
        process::exit(1);
    }

    // allow decompile to handle the rest of main
    if let Err(e) = decompile(input_path, out_path, !lazy, !singlethread, verbose, deobfuscate, !preserve) {
        eprintln!("Error parsing gamedata:\n{}", e);
        process::exit(1);
    }

    if should_pause {
        pause(false);
    }
}

fn decompile(
    in_path: &Path,
    out_path: Option<String>,
    strict: bool,
    multithread: bool,
    verbose: bool,
    deobf_mode: deobfuscate::Mode,
    fix_events: bool,
) -> Result<(), String> {
    // slurp in file contents
    let file = fs::read(&in_path).map_err(|e| format!("Failed to read '{}': {}", in_path.display(), e))?;

    // parse (entire) gamedata
    let logger = if verbose { Some(|msg: &str| println!("{}", msg)) } else { None };
    let mut assets = gm8exe::reader::from_exe(file, logger, strict, multithread) // huge call
        .map_err(|e| format!("Reader error: {}", e))?;

    println!("Successfully parsed game!");

    //Do we want to deobfuscate, yes or no?
    let deobfuscate = match deobf_mode {
        deobfuscate::Mode::On => true,
        deobfuscate::Mode::Off => false,
        deobfuscate::Mode::Auto => {
            assets.backgrounds.iter().flatten().any(|s| s.name.0.is_empty())
                || assets.fonts.iter().flatten().any(|s| s.name.0.is_empty())
                || assets.objects.iter().flatten().any(|s| s.name.0.is_empty())
                || assets.paths.iter().flatten().any(|s| s.name.0.is_empty())
                || assets.rooms.iter().flatten().any(|s| s.name.0.is_empty())
                || assets.sounds.iter().flatten().any(|s| s.name.0.is_empty())
                || assets.sprites.iter().flatten().any(|s| s.name.0.is_empty())
                || assets.timelines.iter().flatten().any(|s| s.name.0.is_empty())
        },
    };
    if deobf_mode == deobfuscate::Mode::Auto && deobfuscate {
        println!("Note: GMK looks obfuscated, so de-obfuscation has been enabled by default");
        println!(" -- you can turn this off with '-d off'");
    }

    fn fix_event(ev: &mut gm8exe::asset::CodeAction) {
        // So far the only broken event type I know of is custom Execute Code actions.
        // We can fix these by changing the act id and lib id to be a default Execute Code action instead.
        if ev.action_kind == 7 && ev.execution_type == 2 {
            // 7 = code block param, 2 = code execution
            ev.id = 603;
            ev.lib_id = 1;
        }
    }

    if fix_events {
        assets
            .objects
            .iter_mut()
            .flatten()
            .flat_map(|x| x.events.iter_mut().flatten())
            .flat_map(|(_, x)| x.iter_mut())
            .for_each(|ev| fix_event(ev));

        assets
            .timelines
            .iter_mut()
            .flatten()
            .flat_map(|x| x.moments.iter_mut().flat_map(|(_, x)| x.iter_mut()))
            .for_each(|ev| fix_event(ev));
    }

    // warn user if they specified .gmk for 8.0 or .gm81 for 8.0
    let out_expected_ext = match assets.version {
        GameVersion::GameMaker8_0 => "gmk",
        GameVersion::GameMaker8_1 => "gm81",
    };
    let out_path = match out_path {
        Some(p) => {
            let path = PathBuf::from(p);
            match (assets.version, path.extension().and_then(|oss| oss.to_str())) {
                (GameVersion::GameMaker8_0, Some(extension @ "gm81"))
                | (GameVersion::GameMaker8_1, Some(extension @ "gmk")) => {
                    println!(
                        concat!(
                            "***WARNING*** You've specified an output file '{}'",
                            "a .{} file, for a {} game.\nYou should use '-o {}.{}' instead, ",
                            "otherwise you won't be able to load the file with GameMaker.",
                        ),
                        path.display(),
                        extension,
                        match assets.version {
                            GameVersion::GameMaker8_0 => "GameMaker 8.0",
                            GameVersion::GameMaker8_1 => "GameMaker 8.1",
                        },
                        path.file_stem().and_then(|oss| oss.to_str()).unwrap_or("filename"),
                        out_expected_ext,
                    );
                },
                _ => (),
            }
            path
        },
        None => {
            let mut path = PathBuf::from(in_path);
            path.set_extension(out_expected_ext);
            path
        },
    };

    if deobfuscate {
        deobfuscate::process(&mut assets);
    }

    let mut gmk = fs::File::create(&out_path)
        .map_err(|e| format!("Failed to create output file '{}': {}", out_path.display(), e))?;

    println!("Writing {} header...", out_expected_ext);
    gmk::write_header(&mut gmk, assets.version, assets.game_id, assets.guid)
        .map_err(|e| format!("Failed to write header: {}", e))?;

    println!("Writing {} settings...", out_expected_ext);
    let ico_file = assets.ico_file_raw.take();
    gmk::write_settings(&mut gmk, &assets.settings, ico_file, assets.version)
        .map_err(|e| format!("Failed to write settings block: {}", e))?;

    println!("Writing {} triggers...", assets.triggers.len());
    gmk::write_asset_list(&mut gmk, &assets.triggers, gmk::write_trigger, assets.version, multithread)
        .map_err(|e| format!("Failed to write triggers: {}", e))?;

    gmk::write_timestamp(&mut gmk).map_err(|e| format!("Failed to write timestamp: {}", e))?;

    println!("Writing {} constants...", assets.constants.len());
    gmk::write_constants(&mut gmk, &assets.constants).map_err(|e| format!("Failed to write constants: {}", e))?;

    println!("Writing {} sounds...", assets.sounds.len());
    gmk::write_asset_list(&mut gmk, &assets.sounds, gmk::write_sound, assets.version, multithread)
        .map_err(|e| format!("Failed to write sounds: {}", e))?;

    println!("Writing {} sprites...", assets.sprites.len());
    gmk::write_asset_list(&mut gmk, &assets.sprites, gmk::write_sprite, assets.version, multithread)
        .map_err(|e| format!("Failed to write sprites: {}", e))?;

    println!("Writing {} backgrounds...", assets.backgrounds.len());
    gmk::write_asset_list(&mut gmk, &assets.backgrounds, gmk::write_background, assets.version, multithread)
        .map_err(|e| format!("Failed to write backgrounds: {}", e))?;

    println!("Writing {} paths...", assets.paths.len());
    gmk::write_asset_list(&mut gmk, &assets.paths, gmk::write_path, assets.version, multithread)
        .map_err(|e| format!("Failed to write paths: {}", e))?;

    println!("Writing {} scripts...", assets.scripts.len());
    gmk::write_asset_list(&mut gmk, &assets.scripts, gmk::write_script, assets.version, multithread)
        .map_err(|e| format!("Failed to write scripts: {}", e))?;

    println!("Writing {} fonts...", assets.fonts.len());
    gmk::write_asset_list(&mut gmk, &assets.fonts, gmk::write_font, assets.version, multithread)
        .map_err(|e| format!("Failed to write fonts: {}", e))?;

    println!("Writing {} timelines...", assets.timelines.len());
    gmk::write_asset_list(&mut gmk, &assets.timelines, gmk::write_timeline, assets.version, multithread)
        .map_err(|e| format!("Failed to write timelines: {}", e))?;

    println!("Writing {} objects...", assets.objects.len());
    gmk::write_asset_list(&mut gmk, &assets.objects, gmk::write_object, assets.version, multithread)
        .map_err(|e| format!("Failed to write objects: {}", e))?;

    println!("Writing {} rooms...", assets.rooms.len());
    gmk::write_asset_list(&mut gmk, &assets.rooms, gmk::write_room, assets.version, multithread)
        .map_err(|e| format!("Failed to write rooms: {}", e))?;

    println!(
        "Writing room editor metadata... (last instance: {}, last tile: {})",
        assets.last_instance_id, assets.last_tile_id
    );
    gmk::write_room_editor_meta(&mut gmk, assets.last_instance_id, assets.last_tile_id)
        .map_err(|e| format!("Failed to write room editor metadata: {}", e))?;

    println!("Writing {} included files...", assets.included_files.len());
    gmk::write_included_files(&mut gmk, &assets.included_files)
        .map_err(|e| format!("Failed to write included files: {}", e))?;

    println!("Writing {} extensions...", assets.extensions.len());
    gmk::write_extensions(&mut gmk, &assets.extensions).map_err(|e| format!("Failed to write extensions: {}", e))?;

    println!("Writing game information...");
    gmk::write_game_information(&mut gmk, &assets.help_dialog)
        .map_err(|e| format!("Failed to write game information: {}", e))?;

    println!("Writing {} library initialization strings...", assets.library_init_strings.len());
    gmk::write_library_init_code(&mut gmk, &assets.library_init_strings)
        .map_err(|e| format!("Failed to write library initialization code: {}", e))?;

    println!("Writing room order ({} rooms)...", assets.room_order.len());
    gmk::write_room_order(&mut gmk, &assets.room_order).map_err(|e| format!("Failed to write room order: {}", e))?;

    println!("Writing resource tree...");
    gmk::write_resource_tree(&mut gmk, &assets).map_err(|e| format!("Failed to write resource tree: {}", e))?;

    println!(
        "Successfully written {} to '{}'",
        out_expected_ext,
        out_path.file_name().and_then(|oss| oss.to_str()).unwrap_or("<INVALID UTF-8>"),
    );

    Ok(())
}
