#![allow(bad_style)]

use super::{dll, state, ID};
use libffi::middle as ffi;
use std::{
    collections::HashMap,
    ffi::{c_void, CStr, CString, OsStr, OsString},
    iter::once,
    os::{
        raw::c_char,
        windows::ffi::{OsStrExt, OsStringExt},
    },
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
    buf_ffi_data: Vec<FfiData>,
    buf_ffi_arg: Vec<ffi::Arg>,

    defs: HashMap<ID, External>,
    id: ID,
}

enum External {
    Dummy(DummyExternal),
    Dll(DllExternal),
}

struct DummyExternal {
    dll: String,
    symbol: String,
    dummy: dll::Value,
    argc: usize,
}

struct DllExternal {
    call_conv: dll::CallConv,
    cif: ffi::Cif,
    code_ptr: ffi::CodePtr,
    type_args: Vec<dll::ValueType>,
    type_return: dll::ValueType,
    dll: String,
    symbol: String,
}

enum FfiData {
    Real(f64),
    Str(*const c_char),
}

impl NativeExternals {
    pub fn new() -> Result<Self, String> {
        Ok(Self { buf_ffi_data: Vec::new(), buf_ffi_arg: Vec::new(), defs: Default::default(), id: 0 })
    }

    pub fn call(&mut self, id: ID, args: &[dll::Value]) -> Result<dll::Value, String> {
        let external = match self.defs.get(&id) {
            Some(x) => x,
            None => return Err(format!("undefined external with id {}", id)),
        };

        match external {
            External::Dummy(dummy) => {
                if args.len() != dummy.argc {
                    Err(format!(
                        "wrong number of arguments passed to dummy '{}' (expected {}, got {})",
                        dummy.symbol,
                        dummy.argc,
                        args.len(),
                    ))
                } else {
                    // most likely this is a sound dll call and we're recording so it will break
                    match dummy.symbol.as_ref() {
                        s
                        @
                        ("SS_IsSoundPlaying"
                        | "SS_IsSoundPaused"
                        | "SS_IsSoundLooping"
                        | "FMODInstanceIsPlaying"
                        | "FMODInstanceGetPaused") => {
                            println!("WARNING: {} called while recording, stuff might break", s)
                        },
                        _ => (),
                    }
                    Ok(dummy.dummy.clone())
                }
            },
            External::Dll(external) => {
                if args.len() != external.type_args.len() {
                    return Err(format!(
                        "wrong number of arguments passed to '{}' (expected {}, got {})",
                        external.symbol,
                        external.type_args.len(),
                        args.len(),
                    ))
                }

                self.buf_ffi_data.clear();
                self.buf_ffi_data.extend(args.iter().zip(external.type_args.iter()).map(|(arg, ty)| match (arg, ty) {
                    (dll::Value::Real(x), dll::ValueType::Real) => FfiData::Real(*x),
                    (dll::Value::Real(_), dll::ValueType::Str) => {
                        FfiData::Str(dll::PascalString::empty().as_ptr().cast())
                    },
                    (dll::Value::Str(x), dll::ValueType::Str) => FfiData::Str(x.as_ptr().cast()),
                    (dll::Value::Str(_), dll::ValueType::Real) => FfiData::Real(0.0),
                }));

                self.buf_ffi_arg.clear();
                self.buf_ffi_arg.extend(self.buf_ffi_data.iter().map(|data| match data {
                    FfiData::Real(x) => ffi::Arg::new(x),
                    FfiData::Str(s) => ffi::Arg::new(s),
                }));

                unsafe {
                    Ok(match external.type_return {
                        dll::ValueType::Real => {
                            dll::Value::Real(external.cif.call::<f64>(external.code_ptr, &self.buf_ffi_arg))
                        },
                        dll::ValueType::Str => dll::Value::Str({
                            let char_ptr = external.cif.call::<*const c_char>(external.code_ptr, &self.buf_ffi_arg);
                            if *char_ptr != 0 {
                                dll::PascalString::new(CStr::from_ptr(char_ptr).to_bytes())
                            } else {
                                dll::PascalString::empty()
                            }
                        }),
                    })
                }
            },
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
            self.defs.insert(
                id,
                External::Dll(DllExternal {
                    cif,
                    code_ptr,
                    type_return,
                    call_conv,
                    type_args: type_args.to_vec(),
                    dll: dll.into(),
                    symbol: symbol.into(),
                }),
            );
            self.id += 1;
            Ok(id)
        }
    }

    pub fn define_dummy(&mut self, dll: &str, symbol: &str, dummy: dll::Value, argc: usize) -> Result<ID, String> {
        let id = self.id;
        self.defs.insert(id, External::Dummy(DummyExternal { dll: dll.into(), symbol: symbol.into(), dummy, argc }));
        self.id += 1;
        Ok(id)
    }

    pub fn free(&mut self, target: &str) -> Result<(), String> {
        self.defs.retain(|_, v| match v {
            External::Dummy(DummyExternal { dll, .. }) => !dll.eq_ignore_ascii_case(target),
            External::Dll(DllExternal { dll, .. }) => !dll.eq_ignore_ascii_case(target),
        });
        Ok(())
    }

    pub fn ss_id(&mut self) -> Result<ID, String> {
        Ok(self.id)
    }

    pub fn ss_set_id(&mut self, next: ID) -> Result<(), String> {
        self.id = next;
        Ok(())
    }

    pub fn ss_query_defs(&mut self) -> Result<(HashMap<ID, self::state::State>, ID), String> {
        Ok((
            self.defs
                .iter()
                .map(|(id, def)| match def {
                    External::Dll(DllExternal { dll, symbol, call_conv, type_args, type_return, .. }) => {
                        (*id, state::State::NormalExternal {
                            dll: dll.clone(),
                            symbol: symbol.clone(),
                            call_conv: *call_conv,
                            type_args: type_args.clone(),
                            type_return: *type_return,
                        })
                    },
                    External::Dummy(DummyExternal { dll, symbol, dummy, argc }) => (*id, state::State::DummyExternal {
                        dll: dll.clone(),
                        symbol: symbol.clone(),
                        dummy: dummy.clone(),
                        argc: *argc,
                    }),
                })
                .collect(),
            self.id,
        ))
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
        0xA11E30FF => (), // above but upx'd (used by gm82snd v1.1.6 and earlier)
        0x04756676 => (), // above but with a fix for unicode paths (used by gm82snd v1.1.7 and later)
        _ => {
            eprintln!("WARNING: Unknown version of GMFMODSimple detected with hash {:#010X}", file_hash);
            eprintln!("GMFMODSimple requires a hack to work, and we weren't able to apply it.");
            eprintln!("The game is likely to crash.");
        },
    }
    Ok(())
}
