use std::{alloc::{self, alloc, dealloc, Layout}, ops::Drop, ptr::NonNull, slice};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CallConv {
    Cdecl,
    Stdcall,
}

pub struct PascalString {
    layout: Layout,
    ptr: Option<NonNull<u8>>,
}

impl PascalString {
    pub fn empty() -> Self {
        Self {
            layout: unsafe { Layout::from_size_align_unchecked(0, 4) },
            ptr: None,
        }
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
                Self {
                    layout,
                    ptr: Some(NonNull::new_unchecked(alloc)),
                }
            } else {
                alloc::handle_alloc_error(layout)
            }
        }
    }

    pub fn as_ptr(&self) -> *const u8 {
        match self.ptr {
            Some(ptr) => ptr.as_ptr(),
            None => &[0u32, 0u32][1] as *const u32 as *const u8,
        }
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

pub enum Value {
    Real(f64),
    Str(PascalString),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ValueType {
    Real,
    Str,
}

impl Value {

}
