#![cfg(all(target_os = "windows", target_arch = "x86"))]

use super::{ArgType, CallConv, ExternalCall};
use crate::gml::Value;
use dll_macros::{external_call, setup_cvalue};
use std::{
    ffi::{CStr, OsStr},
    os::{
        raw::{c_char, c_void},
        windows::ffi::OsStrExt,
    },
    rc::Rc,
};
use winapi::{
    shared::minwindef::HMODULE,
    um::{
        errhandlingapi::GetLastError,
        libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW},
    },
};

impl From<*const c_char> for Value {
    fn from(s: *const c_char) -> Self {
        unsafe { CStr::from_ptr(s).to_string_lossy().to_string().into() }
    }
}

setup_cvalue!(CValue);

impl From<Value> for CValue {
    fn from(v: Value) -> Self {
        match v {
            Value::Real(x) => CValue::Real(x.into()),
            Value::Str(s) => {
                let s = s.as_ref();
                let mut buf = Vec::with_capacity(5 + s.len());
                buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
                buf.extend_from_slice(s.as_bytes());
                buf.push(0);
                CValue::Str(Rc::new(buf))
            },
        }
    }
}

pub struct ExternalImpl {
    dll_handle: HMODULE,
    call: *const c_void,
    call_conv: CallConv,
    res_type: ArgType,
    arg_types: Vec<ArgType>,
}

impl ExternalImpl {
    pub fn new(
        dll_name: &str,
        fn_name: &str,
        call_conv: CallConv,
        res_type: ArgType,
        arg_types: &[ArgType],
    ) -> Result<Self, String> {
        let mut os_dll_name = OsStr::new(dll_name).encode_wide().collect::<Vec<_>>();
        os_dll_name.push(0);
        let mut fn_name = fn_name.to_string();
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
            Ok(Self { dll_handle, call: fun.cast(), call_conv, res_type, arg_types: arg_types.to_vec() })
        }
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, args: &[Value]) -> Value {
        let args = args.iter().map(|v| CValue::from(v.clone())).collect::<Vec<_>>();
        unsafe {
            external_call!(
                self.call,
                args,
                self.call_conv,
                self.res_type,
                self.arg_types.as_slice(),
                CallConv::Cdecl,
                CallConv::Stdcall,
                ArgType::Real,
                ArgType::Str
            )
        }
    }
}

impl Drop for ExternalImpl {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.dll_handle);
        }
    }
}
