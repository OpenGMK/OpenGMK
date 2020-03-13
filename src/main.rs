#![allow(dead_code)] // Shut up.

mod action;
mod asset;
mod atlas;
mod background;
mod game;
mod gml;
mod instance;
mod instancelist;
mod render;
mod tile;
mod types;
mod util;
mod view;

use std::{env, fs, path::Path, process};

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
            return EXIT_FAILURE;
        },
    };

    if args.len() < 2 || matches.opt_present("h") {
        help(&process, opts);
        return EXIT_SUCCESS;
    }

    let strict = matches.opt_present("s");
    let multithread = !matches.opt_present("t");
    let verbose = matches.opt_present("v");
    let input = {
        if matches.free.len() == 1 {
            &matches.free[0]
        } else if matches.free.len() > 1 {
            eprintln!("unexpected second input {}", matches.free[1]);
            return EXIT_FAILURE;
        } else {
            eprintln!("no input file");
            return EXIT_FAILURE;
        }
    };

    let mut file = match fs::read(&input) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("failed to open '{}': {}", input, err);
            return EXIT_FAILURE;
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
            return EXIT_FAILURE;
        },
    };

    let mut components = match game::Game::launch(assets) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Failed to launch game: {}", e);
            return EXIT_FAILURE;
        },
    };

    while !components.renderer.should_close() {
        components.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&components.glfw_events) {
            println!("Got event {:?}", event);
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    components.renderer.set_should_close(true);
                    continue; // So no draw events are fired while the window should be closing
                },
                _ => {},
            }
        }

        components.renderer.set_view(
            0,
            0,
            components.room_width,
            components.room_height,
            0.0,
            0,
            0,
            components.room_width,
            components.room_height,
        );

        // for (_, tile) in components.instance_list.iter_tiles() {
        //     if let Some(Some(background)) = components.assets.backgrounds.get(tile.background_index as usize) {
        //         if let Some(atlas) = &background.atlas_ref {
        //             components.renderer.draw_sprite_partial(
        //                 atlas,
        //                 tile.tile_x as _,
        //                 tile.tile_y as _,
        //                 tile.width as _,
        //                 tile.height as _,
        //                 tile.x,
        //                 tile.y,
        //                 tile.xscale,
        //                 tile.yscale,
        //                 0.0,
        //                 tile.blend,
        //                 tile.alpha,
        //             )
        //         }
        //     }
        // }
        let mut iter = components.instance_list.iter_draw();
        components.instance_list.draw_sort(); // sort by draw order!
        while let Some(idx) = iter.next(&components.instance_list) {
            let instance = components.instance_list.get(idx).expect("uh oh");
            if let Some(Some(sprite)) = components.assets.sprites.get(instance.sprite_index.get() as usize) {
                components.renderer.draw_sprite(
                    &sprite.frames.first().unwrap().atlas_ref,
                    instance.x.get(),
                    instance.y.get(),
                    instance.image_xscale.get(),
                    instance.image_yscale.get(),
                    instance.image_angle.get(),
                    instance.image_blend.get(),
                    instance.image_alpha.get(),
                )
            }
        }
        components.renderer.finish();
    }

    EXIT_SUCCESS
}
