#![cfg(all(target_os = "windows", target_arch = "x86"))]

use super::DefineInfo;
use crate::gml;
use encoding_rs::Encoding;
use shared::dll;
use std::{ffi::CStr, os::raw::c_char};

// shortcut to prevent extra reallocation
impl From<*const c_char> for gml::Value {
    fn from(s: *const c_char) -> Self {
        unsafe { CStr::from_ptr(s).to_string_lossy().to_string().into() }
    }
}

pub struct ExternalImpl(dll::External);

impl ExternalImpl {
    pub fn new(info: &DefineInfo, encoding: &'static Encoding) -> Result<Self, String> {
        let dll_name = info.dll_name.decode(encoding);
        let fn_name = info.fn_name.decode(encoding).into_owned();
        dll::External::new(dll_name.as_ref(), fn_name, info.call_conv, info.res_type, &info.arg_types)
            .map(|external| Self(external))
    }

    pub fn call<I: Iterator<Item = gml::Value>>(&self, args: I) -> Result<gml::Value, String> {
        let args = args
            .map(|v| match v {
                gml::Value::Real(x) => f64::from(x).into(),
                gml::Value::Str(s) => s.as_ref().into(),
            })
            .collect::<Vec<_>>();
        Ok(self.0.call(&args).into())
    }
}
