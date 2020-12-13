pub mod dummy;
pub mod win32;
pub mod win64;

use crate::{
    game::string::RCStr,
    gml::{self, Value},
};
use cfg_if::cfg_if;
use encoding_rs::Encoding;
use serde::{Deserialize, Serialize};
use shared::dll;

pub use shared::dll::{CallConv, ValueType as DLLValueType};

cfg_if! {
    if
    #[cfg(all(target_os = "windows", target_arch = "x86"))] {
        use win32 as platform;
    } else if #[cfg(all(target_os = "windows", target_arch = "x86_64"))] {
        use win64 as platform;
    } else {
        use dummy as platform;
    }
}

pub enum Call {
    DummyNull(dll::ValueType),
    DummyOne,
    DllCall(Box<dyn ExternalCall>),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DefineInfo {
    pub dll_name: RCStr,
    pub fn_name: RCStr,
    pub call_conv: CallConv,
    pub res_type: dll::ValueType,
    pub arg_types: Vec<dll::ValueType>,
}

pub struct External {
    call: Call,
    pub info: DefineInfo,
}

pub trait ExternalCall {
    /// Do any validity checking before calling this function.
    fn call(&self, args: &[Value]) -> Result<Value, String>;
}

impl External {
    pub fn new(info: DefineInfo, disable_sound: bool, encoding: &'static Encoding) -> Result<Self, String> {
        if info.arg_types.len() > 4 && info.arg_types.contains(&dll::ValueType::Str) {
            return Err("DLL functions with more than 4 arguments cannot have string arguments".into())
        }
        if info.arg_types.len() >= 16 {
            return Err("DLL functions can have at most 16 arguments".into())
        }
        let mut dll_name_lower = std::path::Path::new(info.dll_name.decode(encoding).as_ref())
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        dll_name_lower.make_ascii_lowercase();
        let call = match dll_name_lower.as_str() {
            "gmfmodsimple.dll" if disable_sound && info.fn_name.as_ref() == b"FMODSoundAdd" => Call::DummyOne,
            "gmfmodsimple.dll" | "ssound.dll" | "supersound.dll" | "sxms-3.dll" if disable_sound => {
                Call::DummyNull(info.res_type)
            },
            _ => Call::DllCall(Box::new(platform::ExternalImpl::new(&info, encoding)?)),
        };
        Ok(Self { call, info })
    }

    pub fn call(&self, args: &[Value]) -> gml::Result<Value> {
        if args.len() != self.info.arg_types.len() {
            Err(gml::Error::WrongArgumentCount(self.info.arg_types.len(), args.len()))
        } else {
            self.call.call(args)
        }
    }
}

impl Call {
    fn call(&self, args: &[Value]) -> gml::Result<Value> {
        match self {
            Call::DummyNull(res_type) => match res_type {
                dll::ValueType::Real => Ok(0.into()),
                dll::ValueType::Str => Ok("".into()),
            },
            Call::DummyOne => Ok(1.into()),
            Call::DllCall(call) => {
                call.call(args).map_err(|e| gml::Error::FunctionError("external_call".into(), e.into()))
            },
        }
    }
}
