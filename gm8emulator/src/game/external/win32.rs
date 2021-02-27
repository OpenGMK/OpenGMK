#![cfg(all(target_os = "windows", target_arch = "x86"))]

use super::{DefineInfo, ExternalCall};
use crate::{game::string::RCStr, gml};
use encoding_rs::Encoding;
use shared::{dll, dll::ValueType};
use std::{ffi::CStr, os::raw::c_char};

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
    external: dll::External,
    arg_types: Vec<dll::ValueType>,
}

impl ExternalImpl {
    pub fn new(info: &DefineInfo, encoding: &'static Encoding) -> Result<Self, String> {
        let dll_name = info.dll_name.decode(encoding);
        let fn_name = info.fn_name.decode(encoding).into_owned();
        dll::External::new(dll_name.as_ref(), fn_name, info.call_conv, info.res_type, &info.arg_types)
            .map(|external| Self { external, arg_types: info.arg_types.clone() })
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, args: &[gml::Value]) -> Result<gml::Value, String> {
        let args = args.iter().zip(&self.arg_types).map(|(v, t)| to_dll_value(v.clone(), t)).collect::<Vec<_>>();
        Ok(self.external.call(&args).into())
    }
}
