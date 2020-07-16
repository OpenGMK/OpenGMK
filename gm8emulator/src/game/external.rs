pub mod dummy;
pub mod win32;
pub mod win64;

use crate::{
    game::string::RCStr,
    gml::{self, Value},
};
use cfg_if::cfg_if;
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

#[derive(Clone, Serialize, Deserialize)]
pub struct DefineInfo {
    pub dll_name: RCStr,
    pub fn_name: RCStr,
    pub call_conv: CallConv,
    pub res_type: dll::ValueType,
    pub arg_types: Vec<dll::ValueType>,
}

pub struct External {
    call: Box<dyn ExternalCall>,
    pub info: DefineInfo,
}

pub trait ExternalCall {
    /// Do any validity checking before calling this function.
    fn call(&self, args: &[Value]) -> Result<Value, String>;
}

impl External {
    pub fn new(info: DefineInfo) -> Result<Self, String> {
        if info.arg_types.len() > 4 && info.arg_types.contains(&dll::ValueType::Str) {
            return Err("DLL functions with more than 4 arguments cannot have string arguments".into())
        }
        if info.arg_types.len() >= 16 {
            return Err("DLL functions can have at most 16 arguments".into())
        }
        Ok(Self { call: Box::new(platform::ExternalImpl::new(&info)?), info })
    }

    pub fn call(&self, args: &[Value]) -> gml::Result<Value> {
        if args.len() != self.info.arg_types.len() {
            Err(gml::Error::WrongArgumentCount(self.info.arg_types.len(), args.len()))
        } else {
            self.call.call(args).map_err(|e| gml::Error::FunctionError("external_call".into(), e.into()))
        }
    }
}
