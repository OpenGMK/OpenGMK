use gm8x::{GameVersion, reader::GameAssets};
use std::{env, fs, path::Path};

pub mod collision;
pub mod gmk;
pub mod zlib;

fn main() {
    // Set up getopts to parse our command line args
    let args: Vec<String> = env::args().collect();

    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "prints this help message");
    opts.optflag("l", "lazy", "disables various data integrity checks");
    opts.optflag("v", "verbose", "enables verbose output");
    opts.optopt("o", "output", "specify an output file", "mygame.gmk");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            match f {
                getopts::Fail::ArgumentMissing(arg) => println!("Missing argument: {}", arg),
                getopts::Fail::UnrecognizedOption(opt) => println!("Unrecognized option: {}", opt),
                getopts::Fail::OptionMissing(opt) => println!("Missing option: {}", opt),
                getopts::Fail::OptionDuplicated(opt) => println!("Duplicated option: {}", opt),
                getopts::Fail::UnexpectedArgument(arg) => println!("Unexpected argument: {}", arg),
            }
            return;
        }
    };

    // Print this helpful message if no filename was provided, or if -h/--help was specified
    if matches.free.len() == 0 || matches.opt_present("h") {
        println!(
            "{}",
            opts.usage(&format!("Command usage: {} FILENAME [options]", &args[0]))
        );
        println!(
            "Tip: to decompile a game, click and drag it on top of {}.",
            args[0]
        );
        return;
    }

    // Print this slightly less helpful error message if multiple filenames were provided
    if matches.free.len() > 1 {
        println!("Unexpected input: {}", matches.free[1]);
        return;
    }

    // Get our options and then repeat them back to the user
    let lazy = matches.opt_present("l");
    let verbose = matches.opt_present("v");
    let input = &matches.free[0];

    println!("Input file: '{}'", input);
    if lazy {
        println!("Option: lazy mode (--lazy, -l): data integrity checking disabled");
    }
    if verbose {
        println!("Option: verbose mode (--verbose, -v): verbose console output enabled");
    }

    // Figure out the name of our input file minus path
    let input_filename = match Path::new(input).file_name() {
        Some(f) => f.to_string_lossy(),
        None => {
            println!("Failed to open '{}': not a file name", input);
            return;
        }
    };

    // Open the input file and parse it with gm8x
    let mut file = match fs::read(&input) {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to open '{}': {}", input, e);
            return;
        }
    };

    let assets = match gm8x::reader::from_exe(
        &mut file,
        !lazy,
        if verbose {
            Some(|s: &str| println!("[gm8x] {}", s))
        } else {
            None
        },
        None, // dump_dll
        true, // multithread
    ) {
        Ok(a) => a,
        Err(e) => {
            println!("Error parsing exe: {}", e);
            return;
        }
    };

    println!("Successfully parsed assets from '{}'", input);

    // Work out what our output filename should be
    let expected_ext = match assets.version {
        GameVersion::GameMaker8_0 => ".gmk",
        GameVersion::GameMaker8_1 => ".gm81",
    };
    let gmk_filename = match matches.opt_str("o") {
        Some(o) => {
            // warn user if they specified .gmk for 8.1 or .gm81 for 8.0
            let opath = Path::new(&o);
            let ext = opath.extension().and_then(|oss| oss.to_str());
            let stem = opath.file_stem().and_then(|oss| oss.to_str());
            match (ext, assets.version) {
                (Some(ext @ "gm81"), GameVersion::GameMaker8_0)
                | (Some(ext @ "gmk"), GameVersion::GameMaker8_1) => {
                    println!(
                        concat!(
                            "***WARNING*** You've specified an output file '{}', a .{} file, for a {} game. ",
                            "I suggest using '-o {}{}' instead, otherwise you won't be able to load the file with GameMaker.",
                        ),
                        o,
                        ext,
                        match assets.version {
                            GameVersion::GameMaker8_0 => "GameMaker 8.0",
                            GameVersion::GameMaker8_1 => "GameMaker 8.1",
                        },
                        stem.unwrap(),
                        expected_ext
                    );
                }
                _ => (),
            }
            o
        }
        None => format!(
            "{}{}",
            input_filename.trim_end_matches(".exe"),
            expected_ext
        ),
    };

    // write gmk - I wrapped this in a function so we can catch any io errors here.
    match write_gmk(&gmk_filename, assets, verbose) {
        Ok(_) => {
            // successful
            // press any key to continue?
        }
        Err(e) => {
            println!("Error writing gmk: {}", e);
            return;
        }
    }
}

fn write_gmk(filename: &str, assets: GameAssets, verbose: bool) -> std::io::Result<()> {
    println!("Writing to '{}'", filename);

    // Set up a writer to write to our output file
    let mut gmk = fs::File::create(filename)?;

    // Write GMK header
    if verbose {
        println!("Writing GMK header...");
    }
    gmk::write_header(&mut gmk, assets.version, assets.game_id, assets.guid)?;

    // Write settings
    if verbose {
        println!("Writing GMK settings...");
    }
    gmk::write_settings(
        &mut gmk,
        &assets.settings,
        &assets.ico_file_raw,
        assets.version,
    )?;

    // Write triggers
    if verbose {
        println!("Writing {} triggers...", assets.triggers.len());
    }
    gmk::write_asset_list(
        &mut gmk,
        &assets.triggers,
        gmk::write_trigger,
        assets.version,
    )?;
    gmk::write_timestamp(&mut gmk)?;

    // Write constants
    if verbose {
        println!("Writing {} constants...", assets.constants.len());
    }
    gmk::write_constants(&mut gmk, &assets.constants)?;

    // Write sounds
    if verbose {
        println!("Writing {} sounds...", assets.sounds.len());
    }
    gmk::write_asset_list(&mut gmk, &assets.sounds, gmk::write_sound, assets.version)?;

    // Write sprites
    if verbose {
        println!("Writing {} sprites...", assets.sprites.len());
    }
    gmk::write_asset_list(&mut gmk, &assets.sprites, gmk::write_sprite, assets.version)?;

    // Write backgrounds
    if verbose {
        println!("Writing {} backgrounds...", assets.backgrounds.len());
    }
    gmk::write_asset_list(
        &mut gmk,
        &assets.backgrounds,
        gmk::write_background,
        assets.version,
    )?;

    // Write paths
    if verbose {
        println!("Writing {} paths...", assets.paths.len());
    }
    gmk::write_asset_list(&mut gmk, &assets.paths, gmk::write_path, assets.version)?;

    // Write scripts
    if verbose {
        println!("Writing {} scripts...", assets.scripts.len());
    }
    gmk::write_asset_list(&mut gmk, &assets.scripts, gmk::write_script, assets.version)?;

    // Write fonts
    if verbose {
        println!("Writing {} fonts...", assets.fonts.len());
    }
    gmk::write_asset_list(&mut gmk, &assets.fonts, gmk::write_font, assets.version)?;

    // Write timelines
    if verbose {
        println!("Writing {} timelines...", assets.timelines.len());
    }
    gmk::write_asset_list(
        &mut gmk,
        &assets.timelines,
        gmk::write_timeline,
        assets.version,
    )?;

    // Write objects
    if verbose {
        println!("Writing {} objects...", assets.objects.len());
    }
    gmk::write_asset_list(&mut gmk, &assets.objects, gmk::write_object, assets.version)?;

    // Write rooms
    if verbose {
        println!("Writing {} rooms...", assets.rooms.len());
    }
    gmk::write_asset_list(&mut gmk, &assets.rooms, gmk::write_room, assets.version)?;

    // Write room editor metadata
    if verbose {
        println!(
            "Writing room editor metadata (last instance: {}, last tile: {})...",
            assets.last_instance_id, assets.last_tile_id
        );
    }
    gmk::write_room_editor_meta(&mut gmk, assets.last_instance_id, assets.last_tile_id)?;

    // Write included files
    if verbose {
        println!("Writing {} included files...", assets.included_files.len());
    }
    gmk::write_included_files(&mut gmk, &assets.included_files)?;

    // Write extensions
    if verbose {
        println!("Writing {} extensions...", assets.extensions.len());
    }
    gmk::write_extensions(&mut gmk, &assets.extensions)?;

    // Write game information
    if verbose {
        println!("Writing game information...");
    }
    gmk::write_game_information(&mut gmk, &assets.help_dialog)?;

    // Write library initialization code
    if verbose {
        println!(
            "Writing {} library initialization strings...",
            assets.library_init_strings.len()
        );
    }
    gmk::write_library_init_code(&mut gmk, &assets.library_init_strings)?;

    // Write room order
    if verbose {
        println!("Writing room order ({} rooms)...", assets.room_order.len());
    }
    gmk::write_room_order(&mut gmk, &assets.room_order)?;

    // Write resource tree
    if verbose {
        println!("Writing resource tree...");
    }
    gmk::write_resource_tree(&mut gmk, &assets)?;

    Ok(())
}
