//! This is a bridge that allows 64-bit Windows builds of the emulator to call functions from 32-bit DLL files.
//! The emulator passes commands to the bridge using a pipe, which the bridge executes.
//! Obviously, this is to be built as a 32-bit exe and bundled with 64-bit Windows builds of the emulator.

#[cfg(not(all(target_os = "windows", target_arch = "x86")))]
compile_error!("dll-bridge cannot be built for a target other than windows 32-bit");

use shared::{
    dll::{self, CallConv, DefineResult},
    message::MessageStream,
};
use std::io::{self, Read, Write};

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

struct ExternalList(Vec<Option<dll::External>>);

impl ExternalList {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add_external(
        &mut self,
        dll_name: String,
        fn_name: String,
        call_conv: CallConv,
        res_type: dll::ValueType,
        arg_types: Vec<dll::ValueType>,
    ) -> DefineResult {
        let external_id = self.0.len();
        self.0.push(Some(dll::External::new(&dll_name, fn_name, call_conv, res_type, &arg_types)?));
        return Ok(external_id as u32)
    }

    fn call_external(&self, id: u32, args: Vec<dll::Value>) -> dll::Value {
        let external = self.0[id as usize].as_ref().unwrap();
        external.call(&args)
    }

    fn free_external(&mut self, id: u32) {
        self.0[id as usize] = None;
    }
}

fn main() -> io::Result<()> {
    let mut pipe = Pipe { stdin: io::stdin(), stdout: io::stdout() };
    let mut externals = ExternalList::new();
    let mut read_buffer = Vec::new();
    loop {
        match pipe.receive_message::<dll::Message>(&mut read_buffer)? {
            Some(None) => std::thread::yield_now(),
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
