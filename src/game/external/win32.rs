#![cfg(all(target_os = "windows", target_arch = "x86"))]

use super::{ArgType, CallConv, ExternalCall};
use crate::gml::Value;
use std::{
    ffi::{CStr, OsStr},
    mem::transmute,
    os::{
        raw::{c_char, c_void},
        windows::ffi::OsStrExt,
    },
    rc::Rc,
};
use winapi::{
    shared::minwindef::HMODULE,
    um::{
        errhandlingapi::GetLastError,
        libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW},
    },
};

type PStr = *const c_char;

impl From<PStr> for Value {
    fn from(s: PStr) -> Self {
        unsafe { CStr::from_ptr(s).to_string_lossy().to_string().into() }
    }
}

macro_rules! _arg {
    (f64, $v: expr) => {{ f64::from($v) }};
    (PStr, $v: expr) => {{ l }};
}

#[derive(Clone)]
enum CValue {
    Real(f64),
    Str(Rc<Vec<u8>>),
}

impl From<Value> for CValue {
    fn from(v: Value) -> Self {
        match v {
            Value::Real(x) => CValue::Real(x.into()),
            Value::Str(s) => {
                let s = s.as_ref();
                let mut buf = Vec::with_capacity(5 + s.len());
                buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
                buf.extend_from_slice(s.as_bytes());
                buf.push(0);
                CValue::Str(Rc::new(buf))
            },
        }
    }
}

impl From<CValue> for f64 {
    fn from(v: CValue) -> Self {
        match v {
            CValue::Real(x) => x,
            CValue::Str(_) => 0.0,
        }
    }
}

static EMPTY_PSTRING: [c_char; 5] = [0, 0, 0, 0, 0];

impl From<CValue> for PStr {
    fn from(v: CValue) -> Self {
        unsafe {
            match v {
                CValue::Real(_) => EMPTY_PSTRING.as_ptr().offset(4),
                CValue::Str(s) => s.as_ref().as_ptr().offset(4).cast(),
            }
        }
    }
}

macro_rules! call {
    ($fun: expr, $conv: literal, $res: ty) => {{
        transmute::<_, extern $conv fn() -> $res>($fun)().into()
    }};
    ($fun: expr, $conv: literal, $res: ty, [$a0: ty], $a: expr) => {{
        transmute::<_, extern $conv fn($a0) -> $res>($fun)($a[0].clone().into()).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, [$a0: ty, $a1: ty], $a: expr) => {{
        transmute::<_, extern $conv fn($a0, $a1) -> $res>($fun)($a[0].clone().into(), $a[1].clone().into()).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, [$a0: ty, $a1: ty, $a2: ty], $a: expr) => {{
        transmute::<_, extern $conv fn($a0, $a1, $a2) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, [$a0: ty, $a1: ty, $a2: ty, $a3: ty], $a: expr) => {{
        transmute::<_, extern $conv fn($a0, $a1, $a2, $a3) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 5, $a: expr) => {{
        transmute::<_, extern $conv fn(f64, f64, f64, f64, f64) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 6, $a: expr) => {{
        transmute::<_, extern $conv fn(f64, f64, f64, f64, f64, f64) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 7, $a: expr) => {{
        transmute::<_, extern $conv fn(f64, f64, f64, f64, f64, f64, f64) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 8, $a: expr) => {{
        transmute::<_, extern $conv fn(f64, f64, f64, f64, f64, f64, f64, f64) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into(),
            $a[7].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 9, $a: expr) => {{
        transmute::<_, extern $conv fn(f64, f64, f64, f64, f64, f64, f64, f64, f64) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into(),
            $a[7].clone().into(),
            $a[8].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 10, $a: expr) => {{
        transmute::<_, extern $conv fn(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into(),
            $a[7].clone().into(),
            $a[8].clone().into(),
            $a[9].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 11, $a: expr) => {{
        transmute::<_, extern $conv fn(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into(),
            $a[7].clone().into(),
            $a[8].clone().into(),
            $a[9].clone().into(),
            $a[10].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 12, $a: expr) => {{
        transmute::<_, extern $conv fn(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into(),
            $a[7].clone().into(),
            $a[8].clone().into(),
            $a[9].clone().into(),
            $a[10].clone().into(),
            $a[11].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 13, $a: expr) => {{
        transmute::<_, extern $conv fn(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into(),
            $a[7].clone().into(),
            $a[8].clone().into(),
            $a[9].clone().into(),
            $a[10].clone().into(),
            $a[11].clone().into(),
            $a[12].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 14, $a: expr) => {{
        transmute::<_, extern $conv fn(
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64
        ) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into(),
            $a[7].clone().into(),
            $a[8].clone().into(),
            $a[9].clone().into(),
            $a[10].clone().into(),
            $a[11].clone().into(),
            $a[12].clone().into(),
            $a[13].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 15, $a: expr) => {{
        transmute::<_, extern $conv fn(
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64
        ) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into(),
            $a[7].clone().into(),
            $a[8].clone().into(),
            $a[9].clone().into(),
            $a[10].clone().into(),
            $a[11].clone().into(),
            $a[12].clone().into(),
            $a[13].clone().into(),
            $a[14].clone().into()
        ).into()
    }};
    ($fun: expr, $conv: literal, $res: ty, 16, $a: expr) => {{
        transmute::<_, extern $conv fn(
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
            f64,
        ) -> $res>($fun)(
            $a[0].clone().into(),
            $a[1].clone().into(),
            $a[2].clone().into(),
            $a[3].clone().into(),
            $a[4].clone().into(),
            $a[5].clone().into(),
            $a[6].clone().into(),
            $a[7].clone().into(),
            $a[8].clone().into(),
            $a[9].clone().into(),
            $a[10].clone().into(),
            $a[11].clone().into(),
            $a[12].clone().into(),
            $a[13].clone().into(),
            $a[14].clone().into(),
            $a[15].clone().into()
        ).into()
    }};
}

pub struct ExternalImpl {
    dll_handle: HMODULE,
    call: *const c_void,
    call_conv: CallConv,
    res_type: ArgType,
    arg_types: Vec<ArgType>,
}

impl ExternalImpl {
    pub fn new(
        dll_name: &str,
        fn_name: &str,
        call_conv: CallConv,
        res_type: ArgType,
        arg_types: &[ArgType],
    ) -> Result<Self, String> {
        let mut os_dll_name = OsStr::new(dll_name).encode_wide().collect::<Vec<_>>();
        os_dll_name.push(0);
        let mut fn_name = fn_name.to_string();
        fn_name.push('\0');
        unsafe {
            let dll_handle = LoadLibraryW(os_dll_name.as_ptr());
            if dll_handle.is_null() {
                return Err(format!("Failed to load DLL {}! (Code: {:#X})", dll_name, GetLastError()))
            }
            let fun = GetProcAddress(dll_handle, fn_name.as_ptr() as *const c_char);
            if fun.is_null() {
                FreeLibrary(dll_handle);
                return Err(format!(
                    "Failed to load function {} in DLL {}! (Code: {:#X})",
                    fn_name,
                    dll_name,
                    GetLastError()
                ))
            }
            Ok(Self { dll_handle, call: fun.cast(), call_conv, res_type, arg_types: arg_types.to_vec() })
        }
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, args: &[Value]) -> Value {
        let args = args.iter().map(|v| CValue::from(v.clone())).collect::<Vec<_>>();
        unsafe {
            match self.arg_types.len() {
                0 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr),
                },
                1 => match (self.call_conv, self.res_type, self.arg_types[0]) {
                    (CallConv::Cdecl, ArgType::Real, ArgType::Real) => call!(self.call, "cdecl", f64, [f64], args),
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real) => call!(self.call, "cdecl", PStr, [f64], args),
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real) => call!(self.call, "stdcall", f64, [f64], args),
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real) => call!(self.call, "stdcall", PStr, [f64], args),
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str) => call!(self.call, "cdecl", f64, [PStr], args),
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str) => call!(self.call, "cdecl", PStr, [PStr], args),
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str) => call!(self.call, "stdcall", f64, [PStr], args),
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str) => call!(self.call, "stdcall", PStr, [PStr], args),
                },
                2 => match (self.call_conv, self.res_type, self.arg_types[0], self.arg_types[1]) {
                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [PStr, f64], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [PStr, PStr], args)
                    },
                },
                3 => match (self.call_conv, self.res_type, self.arg_types[0], self.arg_types[1], self.arg_types[2]) {
                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [f64, f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [f64, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [f64, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [f64, f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [PStr, f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [PStr, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [PStr, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [PStr, f64, f64], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [f64, PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [f64, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [f64, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [f64, PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [PStr, PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [PStr, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [PStr, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [PStr, PStr, f64], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [f64, f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [f64, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [f64, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [f64, f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [PStr, f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [PStr, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [PStr, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [PStr, f64, PStr], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [f64, PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [f64, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [f64, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [f64, PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [PStr, PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [PStr, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [PStr, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [PStr, PStr, PStr], args)
                    },
                },
                4 => match (
                    self.call_conv,
                    self.res_type,
                    self.arg_types[0],
                    self.arg_types[1],
                    self.arg_types[2],
                    self.arg_types[3],
                ) {
                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [f64, f64, f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [f64, f64, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [f64, f64, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [f64, f64, f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [PStr, f64, f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [PStr, f64, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [PStr, f64, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [PStr, f64, f64, f64], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [f64, PStr, f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [f64, PStr, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [f64, PStr, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [f64, PStr, f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [PStr, PStr, f64, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [PStr, PStr, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [PStr, PStr, f64, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [PStr, PStr, f64, f64], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [f64, f64, PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [f64, f64, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [f64, f64, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [f64, f64, PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [PStr, f64, PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [PStr, f64, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [PStr, f64, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [PStr, f64, PStr, f64], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [f64, PStr, PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [f64, PStr, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [f64, PStr, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [f64, PStr, PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", f64, [PStr, PStr, PStr, f64], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "cdecl", PStr, [PStr, PStr, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", f64, [PStr, PStr, PStr, f64], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real) => {
                        call!(self.call, "stdcall", PStr, [PStr, PStr, PStr, f64], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [f64, f64, f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [f64, f64, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [f64, f64, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [f64, f64, f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [PStr, f64, f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [PStr, f64, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [PStr, f64, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [PStr, f64, f64, PStr], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [f64, PStr, f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [f64, PStr, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [f64, PStr, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [f64, PStr, f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [PStr, PStr, f64, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [PStr, PStr, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [PStr, PStr, f64, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [PStr, PStr, f64, PStr], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [f64, f64, PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [f64, f64, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [f64, f64, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [f64, f64, PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [PStr, f64, PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [PStr, f64, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [PStr, f64, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [PStr, f64, PStr, PStr], args)
                    },

                    (CallConv::Cdecl, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [f64, PStr, PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [f64, PStr, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [f64, PStr, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [f64, PStr, PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", f64, [PStr, PStr, PStr, PStr], args)
                    },
                    (CallConv::Cdecl, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "cdecl", PStr, [PStr, PStr, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Real, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", f64, [PStr, PStr, PStr, PStr], args)
                    },
                    (CallConv::Stdcall, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str, ArgType::Str) => {
                        call!(self.call, "stdcall", PStr, [PStr, PStr, PStr, PStr], args)
                    },
                },
                5 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 5, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 5, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 5, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 5, args),
                },
                6 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 6, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 6, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 6, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 6, args),
                },
                7 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 7, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 7, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 7, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 7, args),
                },
                8 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 8, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 8, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 8, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 8, args),
                },
                9 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 9, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 9, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 9, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 9, args),
                },
                10 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 10, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 10, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 10, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 10, args),
                },
                11 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 11, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 11, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 11, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 11, args),
                },
                12 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 12, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 12, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 12, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 12, args),
                },
                13 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 13, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 13, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 13, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 13, args),
                },
                14 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 14, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 14, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 14, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 14, args),
                },
                15 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 15, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 15, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 15, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 15, args),
                },
                16 => match (self.call_conv, self.res_type) {
                    (CallConv::Cdecl, ArgType::Real) => call!(self.call, "cdecl", f64, 16, args),
                    (CallConv::Cdecl, ArgType::Str) => call!(self.call, "cdecl", PStr, 16, args),
                    (CallConv::Stdcall, ArgType::Real) => call!(self.call, "stdcall", f64, 16, args),
                    (CallConv::Stdcall, ArgType::Str) => call!(self.call, "stdcall", PStr, 16, args),
                },
                _ => unreachable!(),
            }
        }
    }
}

impl Drop for ExternalImpl {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.dll_handle);
        }
    }
}
