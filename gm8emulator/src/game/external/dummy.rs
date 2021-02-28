use super::DefineInfo;
use crate::gml::Value;
use encoding_rs::Encoding;

pub struct ExternalImpl {}

impl ExternalImpl {
    pub fn new(_info: &DefineInfo, _encoding: &'static Encoding) -> Result<Self, String> {
        Err("DLL loading has not been implemented for this platform".into())
    }

    pub fn call<I>(&self, _args: I) -> Result<Value, String> {
        Err("DLL loading has not been implemented for this platform".into())
    }
}
