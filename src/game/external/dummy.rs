use super::{ArgType, CallConv, ExternalCall};
use crate::gml::Value;

pub struct ExternalImpl {}

impl ExternalImpl {
    pub fn new(
        _dll_name: &str,
        _fn_name: &str,
        _call_conv: CallConv,
        _res_type: ArgType,
        _arg_types: &[ArgType],
    ) -> Result<Self, String> {
        Err("DLL loading has not been implemented for this platform".into())
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, _args: &[Value]) -> Value {
        0.into()
    }
}
