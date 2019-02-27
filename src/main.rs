#![feature(asm)] // Usage: xmath
#![feature(try_trait)]

mod assets;
mod game;
mod xmath;

use crate::game::Game;
use std::env;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: gm8emu <exe_path> [--verbose]");
        return Ok(());
    }

    let verbose = if let Some(arg) = args.get(2) {
        arg == "--verbose"
    } else {
        false
    };
    let data = fs::read(&args[1])?;
    let _game = Game::from_exe(data, verbose)?;

    Ok(())
}
