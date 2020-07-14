pub mod dummy;
pub mod win32;
pub mod win64;

use crate::{
    game::string::RCStr,
    gml::{self, Value},
};
use cfg_if::cfg_if;
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

pub struct External {
    call: Box<dyn ExternalCall>,
    pub dll_name: RCStr,
    arg_count: usize,
}

pub trait ExternalCall {
    /// Do any validity checking before calling this function.
    fn call(&self, args: &[Value]) -> Result<Value, String>;
}

impl External {
    pub fn new(
        dll_name: &str,
        fn_name: &str,
        call_conv: dll::CallConv,
        res_type: dll::ValueType,
        arg_types: &[dll::ValueType],
    ) -> Result<Self, String> {
        if arg_types.len() > 4 && arg_types.contains(&dll::ValueType::Str) {
            return Err("DLL functions with more than 4 arguments cannot have string arguments".into())
        }
        if arg_types.len() >= 16 {
            return Err("DLL functions can have at most 16 arguments".into())
        }
        Ok(Self {
            call: Box::new(platform::ExternalImpl::new(dll_name, fn_name, call_conv, res_type, arg_types)?),
            dll_name: dll_name.into(),
            arg_count: arg_types.len(),
        })
    }

    pub fn call(&self, args: &[Value]) -> gml::Result<Value> {
        if args.len() != self.arg_count {
            Err(gml::Error::WrongArgumentCount(self.arg_count, args.len()))
        } else {
            self.call.call(args).map_err(|e| gml::Error::FunctionError("external_call".into(), e.into()))
        }
    }
}
