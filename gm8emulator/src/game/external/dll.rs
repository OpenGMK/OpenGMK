use serde::{de, ser, Deserialize, Serialize};
use std::{
    alloc::{self, alloc, dealloc, Layout},
    fmt,
    ops::Drop,
    ptr::NonNull,
    slice,
};

// message enum is for stuff that doesn't happen one time at startup
pub const PROTOCOL_VERSION: u16 = 0;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum CallConv {
    Cdecl,
    Stdcall,
}

pub struct PascalString {
    layout: Layout,
    len: usize,
    ptr: Option<NonNull<u8>>,
}

unsafe impl Send for PascalString {}
unsafe impl Sync for PascalString {}

impl PascalString {
    pub fn empty() -> Self {
        Self { layout: unsafe { Layout::from_size_align_unchecked(0, 4) }, len: 0, ptr: None }
    }

    pub fn new(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Self::empty()
        }

        assert!(bytes.len() <= u32::max_value() as usize);
        unsafe {
            let layout = Layout::from_size_align_unchecked(4 + bytes.len() + 1, 4);
            let alloc = alloc(layout);
            if !alloc.is_null() {
                *(alloc as *mut [u8; 4]) = (bytes.len() as u32).to_le_bytes();
                slice::from_raw_parts_mut(alloc.add(4), bytes.len()).copy_from_slice(bytes);
                *alloc.add(4 + bytes.len()) = 0x00;
                Self { layout, len: bytes.len(), ptr: Some(NonNull::new_unchecked(alloc)) }
            } else {
                alloc::handle_alloc_error(layout)
            }
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        match self.ptr {
            Some(ptr) => unsafe { ptr.as_ptr().add(4) },
            None => &[0u32, 0u32][1] as *const u32 as *const u8,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

impl Clone for PascalString {
    fn clone(&self) -> Self {
        Self::new(self.as_slice())
    }
}

impl ser::Serialize for PascalString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_bytes(self.as_slice())
    }
}

impl<'de> de::Deserialize<'de> for PascalString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct PascalStringVisitor;
        impl<'vis> de::Visitor<'vis> for PascalStringVisitor {
            type Value = PascalString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a pascal style string allocation")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(PascalString::new(v))
            }
        }

        deserializer.deserialize_bytes(PascalStringVisitor)
    }
}

impl Drop for PascalString {
    fn drop(&mut self) {
        if let Some(ptr) = self.ptr.take() {
            unsafe {
                dealloc(ptr.as_ptr(), self.layout);
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub enum Value {
    Real(f64),
    Str(PascalString),
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ValueType {
    Real,
    Str,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExternalSignature {
    pub dll: String,
    pub symbol: String,
    pub call_conv: CallConv,
    pub type_args: Vec<ValueType>,
    pub type_return: ValueType,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Wow64Message {
    Call(i32, Vec<Value>),
    Define(ExternalSignature),
    Free(i32),

    Stop,
}
