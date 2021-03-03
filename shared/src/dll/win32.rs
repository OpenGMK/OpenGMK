#![cfg(all(target_os = "windows", target_arch = "x86"))]

use crate::dll::{self, CallConv, ValueType};
use std::{
    ffi::OsStr,
    os::{raw::c_char, windows::ffi::OsStrExt},
};
use winapi::{
    shared::minwindef::HMODULE,
    um::{
        errhandlingapi::GetLastError,
        libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW},
    },
};

impl From<ValueType> for libffi::middle::Type {
    fn from(t: ValueType) -> Self {
        match t {
            ValueType::Real => Self::f64(),
            ValueType::Str => Self::pointer(),
        }
    }
}

pub struct External {
    dll_handle: HMODULE,
    codeptr: libffi::middle::CodePtr,
    cif: libffi::middle::Cif,
    res_type: dll::ValueType,
}

impl External {
    pub fn new(
        dll_name: &str,
        mut fn_name: String,
        call_conv: CallConv,
        res_type: dll::ValueType,
        arg_types: &[dll::ValueType],
    ) -> Result<Self, String> {
        let mut os_dll_name = OsStr::new(&dll_name).encode_wide().collect::<Vec<_>>();
        os_dll_name.push(0);
        fn_name.push('\0');
        unsafe {
            let dll_handle = LoadLibraryW(os_dll_name.as_ptr());
            if dll_handle.is_null() {
                return Err(format!("Failed to load DLL {}! (Code: {:#X})", dll_name, GetLastError()))
            }
            let fun = GetProcAddress(dll_handle, fn_name.as_ptr() as *const c_char);
            if fun.is_null() {
                FreeLibrary(dll_handle);
                return Err(format!(
                    "Failed to load function {} in DLL {}! (Code: {:#X})",
                    fn_name,
                    dll_name,
                    GetLastError()
                ))
            }
            if fn_name == "FMODinit\0" {
                dll::apply_fmod_hack(dll_name.as_ref(), dll_handle.cast())?;
            }
            let codeptr = libffi::middle::CodePtr::from_ptr(fun.cast());
            let cif = libffi::middle::Builder::new()
                .args(arg_types.iter().copied().map(|x| x.into()))
                .res(res_type.into())
                .abi(match call_conv {
                    CallConv::Cdecl => 2,   // FFI_MS_CDECL
                    CallConv::Stdcall => 5, // FFI_STDCALL
                })
                .into_cif();
            Ok(Self { dll_handle, codeptr, cif, res_type })
        }
    }

    pub fn call(&self, args: &[dll::Value]) -> dll::Value {
        // we have a reference to all the values, but we need to specifically store pointers to the strings
        enum IntermediateValue {
            Real(f64),
            Str(*const c_char),
        }
        let arg_ptrs = args
            .iter()
            .map(|v| match v {
                dll::Value::Real(x) => IntermediateValue::Real(*x),
                s @ dll::Value::Str(_) => IntermediateValue::Str(s.into()),
            })
            .collect::<Vec<_>>();
        // now we can safely convert them to libffi args
        let args = arg_ptrs
            .iter()
            .map(|v| match v {
                IntermediateValue::Real(x) => libffi::middle::Arg::new(x),
                IntermediateValue::Str(x) => libffi::middle::Arg::new(x),
            })
            .collect::<Vec<_>>();
        unsafe {
            match self.res_type {
                ValueType::Real => self.cif.call::<f64>(self.codeptr, &args).into(),
                ValueType::Str => self.cif.call::<*const c_char>(self.codeptr, &args).into(),
            }
        }
    }
}

impl Drop for External {
    fn drop(&mut self) {
        unsafe {
            if FreeLibrary(self.dll_handle) == 0 {
                eprintln!("Error freeing DLL (code: {:#X})", GetLastError());
            }
        }
    }
}

// evil hack to make fmod not crash, call when loading FMODinit
pub unsafe fn apply_fmod_hack(filename: &str, handle: *mut u8) -> Result<(), String> {
    let file_data = std::fs::read(filename).map_err(|e| format!("Couldn't load FMOD DLL to hash: {}", e))?;
    let file_hash = crc::crc32::checksum_ieee(&file_data);
    if file_hash == 0xC39E3B94 {
        eprintln!("Applying hack for GMFMODSimple with hash {:#X}", file_hash);
        // i think this is a pointer to some sort of struct containing GM8 handles ripped from the main image
        // if it's null it tries to extract them, which obviously doesn't work with the emulator
        // so make it not null : )
        handle.add(0x852d0).write(1);
    } else {
        eprintln!("WARNING: Unknown version of GMFMODSimple detected with hash {:#X}", file_hash);
        eprintln!("GMFMODSimple requires a hack to work, and we weren't able to apply it.");
        eprintln!("The game is likely to crash.");
    }
    Ok(())
}
