use serde::{Deserialize, Serialize};
use std::{ffi::CStr, os::raw::c_char};

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum CallConv {
    Cdecl,
    Stdcall,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    Real,
    Str,
}

#[cfg(all(target_os = "windows", target_arch = "x86"))]
impl From<ValueType> for libffi::middle::Type {
    fn from(t: ValueType) -> Self {
        match t {
            ValueType::Real => Self::f64(),
            ValueType::Str => Self::pointer(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Value {
    Real(f64),
    Str(Vec<u8>),
}

impl From<Value> for f64 {
    fn from(v: Value) -> Self {
        match v {
            Value::Real(x) => x,
            Value::Str(_) => 0.0,
        }
    }
}

impl From<Value> for *const c_char {
    fn from(v: Value) -> Self {
        unsafe {
            match v {
                Value::Real(_) => b"\0\0\0\0\0".as_ptr().offset(4).cast(),
                Value::Str(s) => s.as_ptr().offset(4).cast(),
            }
        }
    }
}

impl From<f64> for Value {
    fn from(x: f64) -> Self {
        Value::Real(x)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        let mut buf = Vec::with_capacity(5 + s.len());
        buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
        buf.extend_from_slice(s.as_bytes());
        buf.push(0);
        Value::Str(buf)
    }
}

impl From<&[u8]> for Value {
    fn from(s: &[u8]) -> Self {
        let mut buf = Vec::with_capacity(5 + s.len());
        buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
        buf.extend_from_slice(s);
        buf.push(0);
        Value::Str(buf)
    }
}

// Value::Str takes a Pascal string with length but we can't rely on the result being one
impl From<*const c_char> for Value {
    fn from(s: *const c_char) -> Self {
        let bytes = unsafe { CStr::from_ptr(s).to_bytes_with_nul() };
        let mut buf = Vec::with_capacity(4 + bytes.len());
        buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        buf.extend_from_slice(bytes);
        Value::Str(buf)
    }
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    Define { dll_name: String, fn_name: String, call_conv: CallConv, res_type: ValueType, arg_types: Vec<ValueType> },
    Call { func_id: u32, args: Vec<Value> },
    Free { func_id: u32 },
}

pub type DefineResult = Result<u32, String>;

// evil hack to make fmod not crash, call when loading FMODinit
#[cfg(all(target_os = "windows", target_arch = "x86"))]
pub unsafe fn apply_fmod_hack(filename: &str, handle: *mut u8) -> Result<(), String> {
    let file_data = std::fs::read(filename).map_err(|e| format!("Couldn't load FMOD DLL to hash: {}", e))?;
    let file_hash = crc::crc32::checksum_ieee(&file_data);
    if file_hash == 0xC39E3B94 {
        eprintln!("Applying hack for GMFMODSimple with hash {:#X}", file_hash);
        // i think this is a pointer to some sort of struct containing GM8 handles ripped from the main image
        // if it's null it tries to extract them, which obviously doesn't work with the emulator
        // so make it not null : )
        handle.add(0x852d0).write(1);
    } else {
        eprintln!("WARNING: Unknown version of GMFMODSimple detected with hash {:#X}", file_hash);
        eprintln!("GMFMODSimple requires a hack to work, and we weren't able to apply it.");
        eprintln!("The game is likely to crash.");
    }
    Ok(())
}
