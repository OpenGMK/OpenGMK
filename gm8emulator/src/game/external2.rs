mod dll;

#[cfg(all(target_os = "windows"))]
mod win32;
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
mod wow64;

use crate::types::ID;
use self::dll::{CallConv, ValueType};

pub enum ExternalManager {
    Dummy(()),
    Emulated(()),
    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    Win32(win32::NativeExternals),
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    Wow64(()),
}

impl ExternalManager {
    #[inline]
    pub fn new(emulate: bool) -> Result<Self, String> {
        Self::new_(emulate)
    }

    #[cfg(all(target_os = "windows", target_arch = "x86"))]
    fn new_(emulate: bool) -> Result<Self, String> {
        Self::Win32(NativeExternals::new())
    }

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    fn new_(emulate: bool) -> Result<Self, String> {
        let _ = emulate;
        todo!()
    }

    #[cfg(not(target_os = "windows"))]
    fn new_(emulate: bool) -> Result<Self, String> {
        let _ = emulate;
        todo!()
    }

    pub fn define(
        &mut self,
        dll: &str,
        symbol: &str,
        call_conv: CallConv,
        type_args: &[ValueType],
        type_return: ValueType,
    ) -> Result<ID, String> {
        let dll = dll.find('\0').map(|x| &dll[..x]).unwrap_or(dll);
        let symbol = symbol.find('\0').map(|x| &symbol[..x]).unwrap_or(symbol);
        match self {
            Self::Dummy(()) => todo!(),
            Self::Emulated(emu) => todo!(),
            #[cfg(all(target_os = "windows", target_arch = "x86"))]
            Self::Win32(win32) => win32.define(dll, symbol, call_conv, type_args, type_return),
            #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
            Self::Wow64(wow64) => todo!(),
        }
    }
}
