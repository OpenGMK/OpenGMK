use gm8exe::{asset::Object, GameAssets};
use std::str;
use std::{fs, process};

fn get_assets() -> GameAssets {
    let in_path = "I just wanna play the Needle game.exe";
    let file = fs::read(in_path).map_err(|e| format!("Failed to read '{}': {}", in_path, e)).unwrap();
    let verbose = false;
    let logger = if verbose { Some(|msg: &str| println!("{}", msg)) } else { None };

    return gm8exe::reader::from_exe(file, logger, true, true).unwrap_or_else(|err| {
        println!("reader error: {err}");
        process::exit(1);
    });
}

fn get_sprite_name(assets: GameAssets, spr_idx: usize) -> String {
    return str::from_utf8(assets.sprites[spr_idx as usize].as_ref().unwrap().name.0.as_ref()).unwrap().to_string();
}

fn main() {
    let assets = get_assets();
    let spr_idx: usize = assets.objects[0].as_ref().unwrap().sprite_index as usize;
    println!("{:?}", get_sprite_name(assets, spr_idx));
}
