#![allow(bad_style)]

use libffi::middle as ffi;
use super::{ID, dll};
use std::{
    collections::HashMap,
    ffi::{c_void, CStr, CString, OsStr, OsString},
    iter::once,
    os::{raw::c_char, windows::ffi::{OsStrExt, OsStringExt}},
};

const FFI_STDCALL: u32 = 2;
const FFI_MS_CDECL: u32 = 5;

type HINSTANCE = *mut HINSTANCE__;
enum HINSTANCE__ {}

type BOOL = i32;
type CHAR = c_char;
type DWORD = u32;
type HANDLE = *mut c_void;
type HMODULE = HINSTANCE;
type SIZE_T = usize;
type WCHAR = u16;

const PAGE_READWRITE: DWORD = 0x04;
const MAX_PATH: usize = 260;

#[link(name = "kernel32")]
extern "system" {
    fn GetLastError() -> DWORD;

    fn LoadLibraryW(lpFileName: *const WCHAR) -> HMODULE;
    fn GetModuleFileNameW(hModule: HMODULE, lpFilename: *mut WCHAR, nSize: DWORD) -> DWORD;
    fn GetProcAddress(hModule: HMODULE, lpProcName: *const CHAR) -> *const c_void;

    fn FlushInstructionCache(hProcess: HANDLE, lpBaseAddress: *const c_void, dwSize: SIZE_T) -> BOOL;
    fn GetCurrentProcess() -> HANDLE;
    fn VirtualProtect(lpAddress: *mut c_void, dwSize: SIZE_T, flNewProtect: DWORD, lpflOldProtect: *mut DWORD) -> BOOL;
}


impl From<dll::ValueType> for ffi::Type {
    fn from(t: dll::ValueType) -> Self {
        match t {
            dll::ValueType::Real => Self::f64(),
            dll::ValueType::Str => Self::pointer(),
        }
    }
}

fn wstrz(s: &str) -> Vec<WCHAR> {
    OsStr::new(s).encode_wide().chain(once(0x00)).collect()
}

pub struct NativeExternals {
    defs: HashMap<ID, External>,
    id: ID,
}

struct External {
    cif: ffi::Cif,
    code_ptr: ffi::CodePtr,
    type_args: Vec<dll::ValueType>,
    type_return: dll::ValueType,

    // for debug only
    symbol: String,
}

impl NativeExternals {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            defs: Default::default(),
            id: 1, // TODO: what's the start id?
        })
    }

    pub fn call(&self, id: ID, args: &[dll::Value]) -> Result<dll::Value, String> {
        let external = match self.defs.get(&id) {
            Some(x) => x,
            None => return Err(format!("undefined external with id {}", id)),
        };

        if args.len() != external.type_args.len() {
            return Err(format!(
                "wrong number of arguments passed to '{}' (expected {}, got {})",
                external.symbol,
                external.type_args.len(),
                args.len(),
            ))
        }

        let arg_ptrs: Vec<*const c_void> = args
            .iter()
            .zip(external.type_args.iter())
            .map(|(arg, ty)| match (arg, ty) {
                (dll::Value::Real(x), dll::ValueType::Real) => x as *const f64 as *const c_void,
                (dll::Value::Real(_), dll::ValueType::Str) => &[0u32, 0u32][1] as *const u32 as *const c_void,
                (dll::Value::Str(x), dll::ValueType::Str) => x.as_ptr().cast(),
                (dll::Value::Str(_), dll::ValueType::Real) => &0.0f64 as *const f64 as *const c_void,
            })
            .collect();

        let ffi_args: Vec<ffi::Arg> = arg_ptrs
            .iter()
            .map(ffi::Arg::new)
            .collect();

        unsafe {
            Ok(match external.type_return {
                dll::ValueType::Real => dll::Value::Real(external.cif.call::<f64>(external.code_ptr, &ffi_args)),
                dll::ValueType::Str => dll::Value::Str({
                    let char_ptr = external.cif.call::<*const c_char>(external.code_ptr, &ffi_args);
                    if *char_ptr != 0 {
                        dll::PascalString::new(CStr::from_ptr(char_ptr).to_bytes())
                    } else {
                        dll::PascalString::empty()
                    }
                }),
            })
        }
    }

    pub fn define(
        &mut self,
        dll: &str,
        symbol: &str,
        call_conv: dll::CallConv,
        type_args: &[dll::ValueType],
        type_return: dll::ValueType,
    ) -> Result<ID, String> {
        unsafe {
            // load dll & function
            let dll_wchar = wstrz(dll);
            let handle = LoadLibraryW(dll_wchar.as_ptr());
            if handle.is_null() {
                return Err(format!("failed to load dll '{}'", dll))
            }
            let symbol_c = CString::new(symbol).unwrap();
            let function = GetProcAddress(handle, symbol_c.as_ptr());
            if function.is_null() {
                return Err(format!("failed to GetProcAddress for '{}' in '{}", symbol, dll))
            }

            // compatibility hacks
            if symbol == "FMODinit" {
                apply_fmod_hack(handle)?;
            }

            let code_ptr = ffi::CodePtr::from_ptr(function);
            let cif = ffi::Builder::new()
                .args(type_args.iter().copied().map(dll::ValueType::into))
                .res(type_return.into())
                .abi(match call_conv {
                    // TODO: For win64, use other constants
                    dll::CallConv::Cdecl => FFI_MS_CDECL,
                    dll::CallConv::Stdcall => FFI_STDCALL,
                })
                .into_cif();

            let id = self.id;
            self.defs.insert(id, External {
                cif, code_ptr, type_return,
                type_args: type_args.to_vec(),
                symbol: symbol.into(),
            });
            self.id += 1;
            Ok(id)
        }
    }

    pub fn free(&mut self, id: ID) -> Result<(), String> {
        if let None = self.defs.remove(&id) {
            Err(format!("id {} was never defined", id))
        } else {
            Ok(())
        }
    }
}

// evil hack to make fmod not crash, call when loading FMODinit
unsafe fn apply_fmod_hack(handle: HMODULE) -> Result<(), String> {
    let filename = {
        // no way to get the length, so try until it works
        let mut fname = Vec::with_capacity(MAX_PATH);
        loop {
            let len = GetModuleFileNameW(handle, fname.as_mut_ptr(), fname.capacity() as _);
            if len == 0 {
                return Err(format!("couldn't get FMOD dll filename: error code {:#X}", GetLastError()))
            } else if len < fname.capacity() as u32 {
                fname.set_len(len as usize);
                break
            } else {
                fname.reserve(len as usize * 2);
            }
        }
        OsString::from_wide(&fname)
    };
    let handle = handle.cast::<u8>();
    let file_data = std::fs::read(filename).map_err(|e| format!("couldn't load FMOD dll to hash: {}", e))?;
    let file_hash = {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&file_data);
        hasher.finalize()
    };
    // TODO: `log` stuff
    match file_hash {
        0xC39E3B94 => {
            // the usual one
            eprintln!("Applying hack for GMFMODSimple with hash {:#X}", file_hash);
            // i think this is a pointer to some sort of struct containing GM8 handles ripped from the main image
            // if it's null it tries to extract them, which obviously doesn't work with the emulator
            // so make it not null : )
            handle.add(0x852d0).write(1);
        },
        0xD914E241 => {
            // the 2009 build
            eprintln!("Applying hack for GMFMODSimple with hash {:#X}", file_hash);
            // it tries to get the address for ds_list_add but this will access violate
            // so inject a RET instruction into the start of its GetProcAddress function
            // not to be confused with win32's GetProcAddress
            // because we're injecting into executable space, we need to mess with memory permissions
            let target_byte = handle.add(0xe440);
            let mut old_protect = 0;
            VirtualProtect(target_byte.cast(), 1, PAGE_READWRITE, &mut old_protect);
            target_byte.write(0xc3);
            VirtualProtect(target_byte.cast(), 1, old_protect, &mut old_protect);
            FlushInstructionCache(GetCurrentProcess(), target_byte.cast(), 1);
        },
        0xB2FEC528 => (), // grix's fork v4.46, it doesn't have any runner hacks so it's safe
        0xA11E30FF => (), // above but upx'd (used by gm82snd v1.1.6 and other versions)
        _ => {
            eprintln!("WARNING: Unknown version of GMFMODSimple detected with hash {:#X}", file_hash);
            eprintln!("GMFMODSimple requires a hack to work, and we weren't able to apply it.");
            eprintln!("The game is likely to crash.");
        },
    }
    Ok(())
}
