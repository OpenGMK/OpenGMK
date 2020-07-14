use super::{CallConv, DLLValueType, ExternalCall};
use crate::gml::Value;

pub struct ExternalImpl {}

impl ExternalImpl {
    pub fn new(
        _dll_name: &str,
        _fn_name: &str,
        _call_conv: CallConv,
        _res_type: DLLValueType,
        _arg_types: &[DLLValueType],
    ) -> Result<Self, String> {
        Err("DLL loading has not been implemented for this platform".into())
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, _args: &[Value]) -> Result<Value, String> {
        Err("DLL loading has not been implemented for this platform".into())
    }
}
