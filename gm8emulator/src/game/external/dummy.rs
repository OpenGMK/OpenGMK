#![allow(dead_code)]

use super::dll;

// win32 stubs

pub struct NativeExternal;

pub struct NativeManager;

impl NativeManager {
    pub fn new() -> Self {
        Self
    }

    pub fn define(&self, _signature: &dll::ExternalSignature) -> Result<NativeExternal, String> {
        Ok(NativeExternal)
    }

    pub fn call(&mut self, _external: &NativeExternal, _args: &[dll::Value]) -> dll::Value {
        panic!("Native externals are not implemented on this platform.");
    }
}

// wow64 stubs

pub struct IpcExternal;

pub struct IpcManager;

impl IpcManager {
    pub fn new() -> Self {
        Self
    }

    pub fn define(&self, _signature: &dll::ExternalSignature) -> Result<IpcExternal, String> {
        Ok(IpcExternal)
    }

    pub fn call(&mut self, _external: &IpcExternal, _args: &[dll::Value]) -> dll::Value {
        panic!("IPC externals are not implemented on this platform.");
    }

    pub fn free(&mut self, _external: IpcExternal) {}
}
