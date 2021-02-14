#![cfg(all(target_os = "windows", target_arch = "x86"))]

use super::{CallConv, DefineInfo, ExternalCall};
use crate::{game::string::RCStr, gml};
use encoding_rs::Encoding;
use shared::{
    dll,
    dll::{Value, ValueType},
};
use std::{
    ffi::{CStr, OsStr},
    os::{raw::c_char, windows::ffi::OsStrExt},
};
use winapi::{
    shared::minwindef::HMODULE,
    um::{
        errhandlingapi::GetLastError,
        libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW},
    },
};

// shortcut to prevent extra reallocation
impl From<*const c_char> for gml::Value {
    fn from(s: *const c_char) -> Self {
        unsafe { CStr::from_ptr(s).to_string_lossy().to_string().into() }
    }
}

fn to_dll_value(value: gml::Value, type_: &ValueType) -> dll::Value {
    match type_ {
        ValueType::Real => f64::from(value).into(),
        ValueType::Str => RCStr::from(value).as_ref().into(),
    }
}

pub struct ExternalImpl {
    dll_handle: HMODULE,
    codeptr: libffi::middle::CodePtr,
    cif: libffi::middle::Cif,
    res_type: dll::ValueType,
    arg_types: Vec<dll::ValueType>,
}

impl ExternalImpl {
    pub fn new(info: &DefineInfo, encoding: &'static Encoding) -> Result<Self, String> {
        let dll_name = info.dll_name.decode(encoding);
        let mut os_dll_name = OsStr::new(dll_name.as_ref()).encode_wide().collect::<Vec<_>>();
        os_dll_name.push(0);
        let mut os_fn_name = info.fn_name.decode(encoding).into_owned();
        os_fn_name.push('\0');
        unsafe {
            let dll_handle = LoadLibraryW(os_dll_name.as_ptr());
            if dll_handle.is_null() {
                return Err(format!("Failed to load DLL {}! (Code: {:#X})", info.dll_name, GetLastError()))
            }
            let fun = GetProcAddress(dll_handle, os_fn_name.as_ptr() as *const c_char);
            if fun.is_null() {
                FreeLibrary(dll_handle);
                return Err(format!(
                    "Failed to load function {} in DLL {}! (Code: {:#X})",
                    info.fn_name,
                    info.dll_name,
                    GetLastError()
                ))
            }
            if os_fn_name == "FMODinit\0" {
                dll::apply_fmod_hack(dll_name.as_ref(), dll_handle.cast())?;
            }
            let codeptr = libffi::middle::CodePtr::from_ptr(fun.cast());
            fn cnv(t: ValueType) -> libffi::middle::Type {
                match t {
                    ValueType::Real => libffi::middle::Type::f64(),
                    ValueType::Str => libffi::middle::Type::pointer(),
                }
            }
            let cif = libffi::middle::Builder::new()
                .args(info.arg_types.iter().copied().map(cnv))
                .res(cnv(info.res_type))
                .abi(match info.call_conv {
                    CallConv::Cdecl => 2,   // FFI_MS_CDECL
                    CallConv::Stdcall => 5, // FFI_STDCALL
                })
                .into_cif();
            Ok(Self { dll_handle, codeptr, cif, res_type: info.res_type, arg_types: info.arg_types.clone() })
        }
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, args: &[gml::Value]) -> Result<gml::Value, String> {
        let args = args.iter().zip(&self.arg_types).map(|(v, t)| to_dll_value(v.clone(), t)).collect::<Vec<_>>();
        let arg_ptrs = args
            .iter()
            .map(|v| match v {
                Value::Real(x) => libffi::middle::Arg::new(x),
                Value::Str(s) => libffi::middle::Arg::new(&s.as_ptr()),
            })
            .collect::<Vec<_>>();
        unsafe {
            Ok(match self.res_type {
                ValueType::Real => self.cif.call::<f64>(self.codeptr, &arg_ptrs).into(),
                ValueType::Str => self.cif.call::<*const c_char>(self.codeptr, &arg_ptrs).into(),
            })
        }
    }
}

impl Drop for ExternalImpl {
    fn drop(&mut self) {
        unsafe {
            if FreeLibrary(self.dll_handle) == 0 {
                eprintln!("Error freeing DLL (code: {:#X})", GetLastError());
            }
        }
    }
}
