#![cfg(all(target_os = "windows", target_arch = "x86"))]

use super::dll;
use libffi::middle as ffi;
use std::{
    ffi::{c_void, CStr, CString, OsStr, OsString},
    iter::once,
    os::{
        raw::c_char,
        windows::ffi::{OsStrExt, OsStringExt},
    },
};

// ffi stuff
const FFI_STDCALL: u32 = 2;
const FFI_MS_CDECL: u32 = 5;

type HINSTANCE = *mut HINSTANCE__;
enum HINSTANCE__ {}

type BOOL = i32;
type CHAR = c_char;
type DWORD = u32;
type HANDLE = *mut c_void;
type HMODULE = HINSTANCE;
#[allow(non_camel_case_types)]
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
    fn FreeLibrary(hModule: HMODULE) -> BOOL;

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

pub struct NativeManager {
    buf_ffi_data: Vec<FfiData>,
    buf_ffi_arg: Vec<ffi::Arg>,
}

pub struct NativeExternal {
    dll_handle: HMODULE,
    cif: ffi::Cif,
    code_ptr: ffi::CodePtr,
    type_return: dll::ValueType,
}

enum FfiData {
    Real(f64),
    Str(*const c_char),
}

impl NativeManager {
    pub fn new() -> Self {
        Self { buf_ffi_data: vec![], buf_ffi_arg: vec![] }
    }

    pub fn define(&self, signature: &dll::ExternalSignature) -> Result<NativeExternal, String> {
        unsafe {
            // load dll & function
            let dll_wchar = wstrz(&signature.dll);
            let dll_handle = LoadLibraryW(dll_wchar.as_ptr());
            if dll_handle.is_null() {
                return Err(format!("failed to load dll '{}'", signature.dll))
            }
            let symbol_c = CString::new(signature.symbol.clone()).unwrap();
            let function = GetProcAddress(dll_handle, symbol_c.as_ptr());
            if function.is_null() {
                FreeLibrary(dll_handle);
                return Err(format!("failed to GetProcAddress for '{}' in '{}", signature.symbol, signature.dll))
            }

            // compatibility hacks
            if signature.symbol == "FMODinit" {
                apply_fmod_hack(dll_handle)?;
            }

            let code_ptr = ffi::CodePtr::from_ptr(function);
            let cif = ffi::Builder::new()
                .args(signature.type_args.iter().copied().map(dll::ValueType::into))
                .res(signature.type_return.into())
                .abi(match signature.call_conv {
                    // TODO: For win64, use other constants
                    dll::CallConv::Cdecl => FFI_MS_CDECL,
                    dll::CallConv::Stdcall => FFI_STDCALL,
                })
                .into_cif();

            Ok(NativeExternal { dll_handle, cif, code_ptr, type_return: signature.type_return })
        }
    }

    pub fn call(&mut self, external: &NativeExternal, args: &[dll::Value]) -> dll::Value {
        // each libffi arg contains a pointer to the actual arg
        // which means when passing char*, you need a pointer to a pointer to the actual text buffer :^)
        // so store the pointers here
        self.buf_ffi_data.clear();
        self.buf_ffi_data.extend(args.iter().map(|arg| match arg {
            dll::Value::Real(x) => FfiData::Real(*x),
            dll::Value::Str(x) => FfiData::Str(x.as_ptr().cast()),
        }));

        // the args that actually get given to libffi
        self.buf_ffi_arg.clear();
        self.buf_ffi_arg.extend(self.buf_ffi_data.iter().map(|data| match data {
            FfiData::Real(x) => ffi::Arg::new(x),
            FfiData::Str(s) => ffi::Arg::new(s),
        }));

        unsafe {
            match external.type_return {
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
            }
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

impl Drop for NativeExternal {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.dll_handle);
        }
    }
}
