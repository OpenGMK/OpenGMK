pub mod dll;
pub mod state;

#[cfg(all(target_os = "windows"))]
mod win32;
#[cfg(all(target_os = "windows"))]
use self::win32::NativeExternals;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
mod wow64;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
use self::wow64::IpcExternals;

use self::dll::{CallConv, ValueType};
use crate::{game::PlayType, types::ID};
use std::{collections::HashMap, path::Path};

pub enum ExternalManager {
    Emulated(()),
    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    Win32(win32::NativeExternals),
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    Wow64(wow64::IpcExternals),
}

macro_rules! dispatch {
    ($em:expr, $f:ident ( $($arg:ident),* $(,)? ) ) => {
        match $em {
            Self::Emulated(_emu) => todo!(),
            #[cfg(all(target_os = "windows", target_arch = "x86"))]
            Self::Win32(win32) => win32.$f($($arg),*),
            #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
            Self::Wow64(wow64) => wow64.$f($($arg),*),
        }
    };
}

impl ExternalManager {
    #[inline]
    pub fn new(emulate: bool) -> Result<Self, String> {
        if emulate { todo!() } else { Self::new_native() }
    }

    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    fn new_native() -> Result<Self, String> {
        NativeExternals::new().map(Self::Win32)
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    fn new_native() -> Result<Self, String> {
        IpcExternals::new().map(Self::Wow64)
    }

    #[cfg(not(target_os = "windows"))]
    fn new_native() -> Result<Self, String> {
        let _ = emulate;
        todo!()
    }

    pub fn call(&mut self, id: ID, args: &[dll::Value]) -> Result<dll::Value, String> {
        dispatch!(self, call(id, args))
    }

    pub fn define(
        &mut self,
        dll: &str,
        symbol: &str,
        call_conv: CallConv,
        type_args: &[ValueType],
        type_return: ValueType,
    ) -> Result<ID, String> {
        // Akin to `LoadLibraryW` & `GetProcAddress`, pretend it's always null terminated.
        let dll = dll.find('\0').map(|x| &dll[..x]).unwrap_or(dll);
        let symbol = symbol.find('\0').map(|x| &symbol[..x]).unwrap_or(symbol);
        dispatch!(self, define(dll, symbol, call_conv, type_args, type_return))
    }

    pub fn define_dummy(&mut self, dll: &str, symbol: &str, dummy: dll::Value, argc: usize) -> Result<ID, String> {
        dispatch!(self, define_dummy(dll, symbol, dummy, argc))
    }

    pub fn free(&mut self, dll: &str) -> Result<(), String> {
        dispatch!(self, free(dll))
    }

    pub fn ss_id(&mut self) -> Result<ID, String> {
        dispatch!(self, ss_id())
    }

    pub fn ss_set_id(&mut self, next: ID) -> Result<(), String> {
        dispatch!(self, ss_set_id(next))
    }

    pub fn ss_query_defs(&mut self) -> Result<(HashMap<ID, self::state::State>, ID), String> {
        dispatch!(self, ss_query_defs())
    }
}

pub fn should_dummy(dll: &str, sym: &str, play_type: PlayType) -> Option<dll::Value> {
    let dll = Path::new(dll).file_name().and_then(|oss| oss.to_str()).unwrap_or(dll);

    let mut dummy = None;
    if play_type == PlayType::Record {
        if dll.eq_ignore_ascii_case("gmfmodsimple.dll") {
            if sym == "FMODSoundAdd" {
                dummy = Some(dll::Value::Real(1.0));
            } else {
                dummy = Some(dll::Value::Real(0.0));
            }
        } else if dll.eq_ignore_ascii_case("ssound.dll") || dll.eq_ignore_ascii_case("supersound.dll") {
            if sym == "SS_Init" {
                dummy = Some(dll::Value::Str(dll::PascalString::new(b"Yes")));
            } else {
                dummy = Some(dll::Value::Real(0.0));
            }
        } else if dll.eq_ignore_ascii_case("sgaudio.dll") || dll.eq_ignore_ascii_case("sxms-3.dll") {
            dummy = Some(dll::Value::Real(0.0));
        } else if dll.eq_ignore_ascii_case("caster.dll") {
            if sym == "caster_error_message" || sym == "caster_version" {
                dummy = Some(dll::Value::Str(dll::PascalString::new(b"")));
            } else {
                dummy = Some(dll::Value::Real(0.0));
            }
        }
    }

    if dll.eq_ignore_ascii_case("gmeffect_0.1.dll") {
        // TODO: don't
        // ^ floogle's original comment, whatever it may mean
        dummy = Some(dll::Value::Real(0.0));
    }

    dummy
}
