use dll_macros::external_call;
use shared::{
    dll::{self, CallConv, DefineResult},
    message::MessageStream,
};
use std::{
    ffi::OsStr,
    io::{self, Read, Write},
    os::{
        raw::{c_char, c_void},
        windows::ffi::OsStrExt,
    },
};
use winapi::{
    shared::minwindef::HMODULE,
    um::{
        errhandlingapi::GetLastError,
        libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW},
    },
};

struct Pipe {
    stdin: io::Stdin,
    stdout: io::Stdout,
}

impl Read for Pipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stdin.read(buf)
    }
}

impl Write for Pipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}

struct External {
    dll_handle: HMODULE,
    call: *const c_void,
    call_conv: CallConv,
    res_type: dll::ValueType,
    arg_types: Vec<dll::ValueType>,
}

struct ExternalList(Vec<Option<External>>);

impl ExternalList {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add_external(
        &mut self,
        dll_name: String,
        mut fn_name: String,
        call_conv: CallConv,
        res_type: dll::ValueType,
        arg_types: Vec<dll::ValueType>,
    ) -> DefineResult {
        let mut os_dll_name = OsStr::new(&dll_name).encode_wide().collect::<Vec<_>>();
        os_dll_name.push(0);
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
            if fn_name == "FMODinit\0" {
                dll::apply_fmod_hack(&dll_name, dll_handle.cast())?;
            }
            let external_id = self.0.len();
            self.0.push(Some(External { dll_handle, call: fun.cast(), call_conv, res_type, arg_types }));
            return Ok(external_id as u32)
        }
    }

    fn call_external(&self, id: u32, args: Vec<dll::Value>) -> dll::Value {
        let external = self.0[id as usize].as_ref().unwrap();
        unsafe {
            external_call!(
                external.call,
                args,
                external.call_conv,
                external.res_type,
                external.arg_types.as_slice(),
                CallConv::Cdecl,
                CallConv::Stdcall,
                dll::ValueType::Real,
                dll::ValueType::Str
            )
        }
    }

    fn free_external(&mut self, id: u32) {
        if let Some(Some(external)) = self.0.get(id as usize) {
            unsafe {
                FreeLibrary(external.dll_handle);
            }
        }
        self.0[id as usize] = None;
    }
}

fn main() -> io::Result<()> {
    let mut pipe = Pipe { stdin: io::stdin(), stdout: io::stdout() };
    let mut externals = ExternalList::new();
    let mut read_buffer = Vec::new();
    loop {
        match pipe.receive_message::<dll::Message>(&mut read_buffer)? {
            Some(None) => (),
            Some(Some(m)) => match m {
                dll::Message::Define { dll_name, fn_name, call_conv, res_type, arg_types } => {
                    pipe.send_message(externals.add_external(dll_name, fn_name, call_conv, res_type, arg_types))?;
                    pipe.flush()?;
                },
                dll::Message::Call { func_id, args } => {
                    pipe.send_message(externals.call_external(func_id, args))?;
                    pipe.flush()?;
                },
                dll::Message::Free { func_id } => {
                    externals.free_external(func_id);
                },
            },
            None => return Ok(()),
        }
    }
}
