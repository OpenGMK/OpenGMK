use gm8exe::asset::PascalString;
use gm8exe::{asset::Object, GameAssets};
use std::collections::HashSet;
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

fn get_sprite_name(assets: &GameAssets, spr_idx: usize) -> String {
    return pascal_string_to_string(&assets.sprites[spr_idx as usize].as_ref().unwrap().name).to_string();
}

fn pascal_string_to_string(s: &PascalString) -> &str {
    return str::from_utf8(&s.0.as_ref()).unwrap();
}

fn main() {
    let mut bruteforcer_objects: HashSet<&str> = HashSet::<&str>::new();
    bruteforcer_objects.insert("spikeUp");
    bruteforcer_objects.insert("spikeDown");
    bruteforcer_objects.insert("spikeLeft");
    bruteforcer_objects.insert("spikeRight");

    let assets = get_assets();
    for obj in &assets.objects {
        let object_name = obj.as_ref().map_or("no object", |o| pascal_string_to_string(&o.name));
        if bruteforcer_objects.contains(object_name) {
            println!("object name is {}", object_name);
            let spr_idx: usize = obj
                .as_ref()
                .unwrap_or(&assets.objects[0].as_ref().unwrap())
                .sprite_index
                .try_into()
                .unwrap_or_default();
            println!("sprite index is {}", spr_idx);
            println!("sprite name is {}", get_sprite_name(&assets, spr_idx));
            println!("");
        }
    }
}
