#![cfg(all(target_os = "windows", target_arch = "x86"))]

use super::{CallConv, DefineInfo, ExternalCall};
use crate::gml;
use dll_macros::external_call;
use encoding_rs::Encoding;
use shared::dll;
use std::{
    ffi::{CStr, OsStr},
    os::{
        raw::{c_char, c_void},
        windows::ffi::OsStrExt,
    },
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

impl From<gml::Value> for dll::Value {
    fn from(v: gml::Value) -> Self {
        match v {
            gml::Value::Real(x) => dll::Value::Real(x.into()),
            gml::Value::Str(s) => s.as_ref().into(),
        }
    }
}

pub struct ExternalImpl {
    dll_handle: HMODULE,
    call: *const c_void,
    call_conv: CallConv,
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
            Ok(Self {
                dll_handle,
                call: fun.cast(),
                call_conv: info.call_conv,
                res_type: info.res_type,
                arg_types: info.arg_types.clone(),
            })
        }
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, args: &[gml::Value]) -> Result<gml::Value, String> {
        let args = args.iter().map(|v| dll::Value::from(v.clone())).collect::<Vec<_>>();
        unsafe {
            Ok(external_call!(
                self.call,
                args,
                self.call_conv,
                self.res_type,
                self.arg_types.as_slice(),
                CallConv::Cdecl,
                CallConv::Stdcall,
                dll::ValueType::Real,
                dll::ValueType::Str
            ))
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
