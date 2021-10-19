mod gm82core;
mod surface_fix;

use super::dll;
use crate::gml::Function;
use byteorder::{ReadBytesExt, LE};
use phf::phf_map;
use std::{
    fs::File,
    io::{self, Seek, SeekFrom},
    path::Path,
};

type FunctionMap = phf::Map<&'static str, Function>;

const BY_TIME: phf::Map<u32, &'static FunctionMap> = phf_map! {
    0x4B3CFA95_u32 => &surface_fix::FUNCTIONS, // surface_fix 1.2
    0x4BDC4543_u32 => &surface_fix::FUNCTIONS, // surface_fix 1.2.1
    0x4D5B12C2_u32 => &surface_fix::FUNCTIONS, // surface_fix 1.2.2
    0x60CA6584_u32 => &gm82core::FUNCTIONS,    // gm82core 1.2.5
    0x60CE8206_u32 => &gm82core::FUNCTIONS,    // gm82core 1.2.6
};

fn get_pe_timestamp(path: &Path) -> io::Result<u32> {
    let mut dll = File::open(path)?;
    // verify MZ header
    if dll.read_u16::<LE>()? != 0x5a4d {
        return Err(io::ErrorKind::InvalidData.into())
    }
    // find PE header
    dll.seek(SeekFrom::Start(0x3c))?;
    let pe_start = u64::from(dll.read_u32::<LE>()?);
    // verify PE header
    dll.seek(SeekFrom::Start(pe_start))?;
    if dll.read_u32::<LE>()? != 0x4550 {
        return Err(io::ErrorKind::InvalidData.into())
    }
    // get timestamp
    dll.seek(SeekFrom::Start(pe_start + 8))?;
    dll.read_u32::<LE>()
}

fn get_by_timestamp(path: &Path) -> Option<&FunctionMap> {
    BY_TIME.get(&get_pe_timestamp(path).ok()?).copied()
}

pub fn find_emulated(signature: &dll::ExternalSignature) -> Option<Function> {
    let dll = Path::new(&signature.dll);
    if let Some(funcs) = get_by_timestamp(dll) { funcs.get(&signature.symbol).copied() } else { None }
}
