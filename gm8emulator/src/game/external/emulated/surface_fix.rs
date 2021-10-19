use super::FunctionMap;
use crate::{
    game::Game,
    gml::{Function, Result, Value},
};
use phf::phf_map;

pub fn fix_surfaces(game: &mut Game, _args: &[Value]) -> Result<Value> {
    game.surface_fix = true;
    Ok(Default::default())
}

pub fn clear_depth_buffer(_game: &mut Game, _args: &[Value]) -> Result<Value> {
    unimplemented!("called unimplemented SurfaceFix function ClearDepthBuffer")
}

pub fn get_current_surface(game: &Game, _args: &[Value]) -> Result<Value> {
    Ok(game.surface_target.unwrap_or(-1).into())
}

pub fn surface_to_string(_game: &mut Game, _args: &[Value]) -> Result<Value> {
    unimplemented!("called unimplemented SurfaceFix function SurfaceToString")
}

pub fn surface_from_string(_game: &mut Game, _args: &[Value]) -> Result<Value> {
    unimplemented!("called unimplemented SurfaceFix function SurfaceFromString")
}

pub fn write_surface_to_binary_file(_game: &mut Game, _args: &[Value]) -> Result<Value> {
    unimplemented!("called unimplemented SurfaceFix function WriteSurfaceToBinaryFile")
}

pub fn read_surface_from_binary_file(_game: &mut Game, _args: &[Value]) -> Result<Value> {
    unimplemented!("called unimplemented SurfaceFix function ReadSurfaceFromBinaryFile")
}

pub fn change_depth_buffer(_game: &mut Game, _args: &[Value]) -> Result<Value> {
    unimplemented!("called unimplemented SurfaceFix function ChangeDepthBuffer")
}

pub fn enable_depth_writing(_game: &mut Game, _args: &[Value]) -> Result<Value> {
    unimplemented!("called unimplemented SurfaceFix function EnableDepthWriting")
}

pub const FUNCTIONS: FunctionMap = phf_map! {
    "FixSurfaces" => Function::Engine(fix_surfaces),
    "ClearDepthBuffer" => Function::Engine(clear_depth_buffer),
    "GetCurrentSurface" => Function::Constant(get_current_surface),
    "SurfaceToString" => Function::Engine(surface_to_string),
    "SurfaceFromString" => Function::Engine(surface_from_string),
    "WriteSurfaceToBinaryFile" => Function::Engine(write_surface_to_binary_file),
    "ReadSurfaceFromBinaryFile" => Function::Engine(read_surface_from_binary_file),
    "ChangeDepthBuffer" => Function::Engine(change_depth_buffer),
    "EnableDepthWriting" => Function::Engine(enable_depth_writing),
};
