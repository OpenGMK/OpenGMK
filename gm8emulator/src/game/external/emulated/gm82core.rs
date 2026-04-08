use super::FunctionMap;
use crate::{
    game::Game,
    gml::{runtime, Function, Result, Value},
};
use phf::phf_map;

pub fn resize_backbuffer(game: &mut Game, args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(runtime::Error::WrongArgumentCount(2, args.len()))
    }
    let width: u32 = args[0].clone().into();
    let height: u32 = args[1].clone().into();
    game.renderer.resize_framebuffer(width, height, false);
    Ok(Default::default())
}

pub const FUNCTIONS: FunctionMap = phf_map! {
    "resize_backbuffer" => Function::Engine(resize_backbuffer),
};
