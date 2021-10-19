use super::FunctionMap;
use crate::{
    game::Game,
    gml::{Function, Result, Value},
};
use phf::phf_map;

pub fn resize_backbuffer(game: &mut Game, args: &[Value]) -> Result<Value> {
    let width = args[0].clone().into();
    let height = args[1].clone().into();
    game.renderer.resize_framebuffer(width, height, false);
    Ok(Default::default())
}

pub const FUNCTIONS: FunctionMap = phf_map! {
    "resize_backbuffer" => Function::Engine(resize_backbuffer),
};
