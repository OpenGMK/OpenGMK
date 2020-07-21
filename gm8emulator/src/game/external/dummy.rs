use super::{DefineInfo, ExternalCall};
use crate::gml::Value;

pub struct ExternalImpl {}

impl ExternalImpl {
    pub fn new(_info: &DefineInfo) -> Result<Self, String> {
        Err("DLL loading has not been implemented for this platform".into())
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, _args: &[Value]) -> Result<Value, String> {
        Err("DLL loading has not been implemented for this platform".into())
    }
}
