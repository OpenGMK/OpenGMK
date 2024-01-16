#![feature(slice_group_by)]
use gm8exe::{
    asset::{sprite::CollisionMap, Object, PascalString, Sprite},
    GameAssets,
};
use std::{
    collections::{HashMap, HashSet},
    fs, process, str,
};

fn get_assets() -> GameAssets {
    let in_path = "I just wanna play the Needle game.exe";
    let in_path = "Amaranth Needle.exe";
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
fn get_collider(assets: &GameAssets, spr_idx: usize) -> &CollisionMap {
    return &assets.sprites[spr_idx].as_ref().unwrap().colliders[0];
}

fn pascal_string_to_string(s: &PascalString) -> &str {
    return str::from_utf8(&s.0.as_ref()).unwrap();
}

fn main() {
    let mut bruteforcer_objects: HashSet<&str> = HashSet::<&str>::new();
    bruteforcer_objects.insert("block");
    bruteforcer_objects.insert("warp");
    bruteforcer_objects.insert("spikeUp");
    bruteforcer_objects.insert("spikeDown");
    bruteforcer_objects.insert("spikeLeft");
    bruteforcer_objects.insert("spikeRight");
    bruteforcer_objects.insert("minispikeUp");
    bruteforcer_objects.insert("minispikeDown");
    bruteforcer_objects.insert("minispikeLeft");
    bruteforcer_objects.insert("minispikeRight");
    bruteforcer_objects.insert("deliciousFruit");
    bruteforcer_objects.insert("movingPlatform");

    let mut bruteforcer_object_map: HashMap<(&Box<[bool]>, bool), &str> = HashMap::<(&Box<[bool]>, bool), &str>::new();
    let mut game_object_map: HashMap<&str, Vec<&str>> = HashMap::<&str, Vec<&str>>::new();

    let assets = get_assets();
    assets.objects.iter().flatten().for_each(|obj| {
        let object_name = pascal_string_to_string(&obj.name);
        if bruteforcer_objects.contains(object_name) {
            let spr_idx: usize = obj.sprite_index.try_into().unwrap_or_default();
            let collider = get_collider(&assets, spr_idx);
            bruteforcer_object_map.insert((&collider.data, obj.solid), object_name);
            let mask_idx: i32 = obj.mask_index;

            println!("object name is {}", object_name);
            println!("sprite index is {}", spr_idx);
            println!("mask index is {}", mask_idx);
            println!("sprite name is {}", get_sprite_name(&assets, spr_idx));
            println!("collider is {:?}", collider.data);
            println!("");
        }
    });
    assets.objects.iter().flatten().for_each(|obj| {
        let object_name = pascal_string_to_string(&obj.name);
        let spr_idx: usize = obj.sprite_index.try_into().unwrap_or_default();
        let collider = get_collider(&assets, spr_idx);
        if let Some(alias) = bruteforcer_object_map.get(&(&collider.data, obj.solid)) {
            game_object_map.entry(alias).and_modify(|list| list.push(object_name)).or_insert(vec![object_name]);
        }
    });

    println!("the bruteforcer objects in game {:?}", game_object_map);

    println!("the objects ingame clustered");
    let mut existing_objects = assets.objects.iter().flatten().collect::<Vec<&Box<Object>>>();
    existing_objects.sort_unstable_by_key(|obj| {
        let spr_idx: usize = obj.sprite_index.try_into().unwrap_or_default();
        let collider = get_collider(&assets, spr_idx);
        (&collider.data, obj.solid, obj.parent_index)
    });
    existing_objects[..]
        .group_by(|obj1, obj2| {
            let spr_idx1: usize = obj1.sprite_index.try_into().unwrap_or_default();
            let collider1 = get_collider(&assets, spr_idx1);
            let spr_idx2: usize = obj2.sprite_index.try_into().unwrap_or_default();
            let collider2 = get_collider(&assets, spr_idx2);
            (&collider1.data, obj1.solid, obj1.parent_index) == (&collider2.data, obj2.solid, obj2.parent_index)
        })
        .for_each(|group| {
            println!();
            println!();
            group.iter().for_each(|o| print!("{} ", o.name))
        });
    println!("the sprites ingame clustered");
    let mut existing_sprites =
        assets.sprites.iter().flatten().filter(|s| s.colliders.first().is_some()).collect::<Vec<&Box<Sprite>>>();
    existing_sprites.sort_unstable_by_key(|sprite| &sprite.colliders[0].data);
    existing_sprites[..]
        .group_by(|sprite1, sprite2| {
            let collider1 = &sprite1.colliders[0];

            let collider2 = &sprite2.colliders[0];
            &collider1.data == &collider2.data
        })
        .for_each(|group| {
            println!();
            println!();
            group.iter().for_each(|s| print!("{} ", s.name))
        });
}
