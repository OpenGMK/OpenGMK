use super::{dll, state, ID};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use serde::de;
use std::{
    collections::HashMap,
    env,
    io::{Read, Write},
    ops::Drop,
    process,
};

const PROCESS_DEFAULT_NAME: &str = "gm8emulator-wow64.exe";
const PROCESS_ENV_OVERRIDE: &str = "OPENGMK_WOW64_BINARY";

pub struct IpcExternals {
    child: process::Child,
    msgbuf: Vec<u8>,
    stdin: process::ChildStdin,
    stdout: process::ChildStdout,
}

impl IpcExternals {
    pub fn new() -> Result<Self, String> {
        let mut process_path = env::current_exe().expect("failed to query path to current executable");
        process_path.set_file_name(match env::var_os(PROCESS_ENV_OVERRIDE) {
            Some(name) => name,
            None => PROCESS_DEFAULT_NAME.into(),
        });
        if !process_path.exists() {
            panic!("{} was not found, please extract it into your gm8emulator directory", PROCESS_DEFAULT_NAME);
        }
        let mut child = process::Command::new(process_path)
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn child process: {}", e))?;
        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        Ok(Self { child, msgbuf: Vec::new(), stdin, stdout })
    }

    pub fn call(&mut self, id: ID, args: &[dll::Value]) -> Result<dll::Value, String> {
        self.send(dll::Wow64Message::Call(id, args.to_vec()))
    }

    pub fn define(
        &mut self,
        dll: &str,
        symbol: &str,
        call_conv: dll::CallConv,
        type_args: &[dll::ValueType],
        type_return: dll::ValueType,
    ) -> Result<ID, String> {
        self.send(dll::Wow64Message::Define(dll.into(), symbol.into(), call_conv, type_args.into(), type_return))
    }

    pub fn define_dummy(&mut self, dll: &str, symbol: &str, dummy: dll::Value, argc: usize) -> Result<ID, String> {
        self.send(dll::Wow64Message::DefineDummy(dll.into(), symbol.into(), dummy, argc))
    }

    pub fn free(&mut self, dll: &str) -> Result<(), String> {
        self.send(dll::Wow64Message::Free(dll.into()))
    }

    pub fn ss_id(&mut self) -> Result<ID, String> {
        self.send(dll::Wow64Message::GetNextId)
    }

    pub fn ss_set_id(&mut self, next: ID) -> Result<(), String> {
        self.send(dll::Wow64Message::SetNextId(next))
    }

    pub fn ss_query_defs(&mut self) -> Result<(HashMap<ID, self::state::State>, ID), String> {
        self.send(dll::Wow64Message::QueryDefs)
    }

    fn send<T>(&mut self, message: dll::Wow64Message) -> Result<T, String>
    where
        T: for<'de> de::Deserialize<'de>,
    {
        self.msgbuf.clear();
        bincode::serialize_into(&mut self.msgbuf, &message).expect("failed to serialize message (client)");
        assert!(self.msgbuf.len() <= u32::max_value() as usize);
        self.stdin
            .write_u32::<LE>(self.msgbuf.len() as u32)
            .and_then(|_| self.stdin.write_all(self.msgbuf.as_slice()))
            .and_then(|_| self.stdin.flush())
            .map_err(|io| format!("failed to write to child stdin: {}", io))?;
        self.stdout
            .read_u32::<LE>()
            .and_then(|len| {
                let length = len as usize;
                self.msgbuf.clear();
                self.msgbuf.reserve(length);
                unsafe { self.msgbuf.set_len(length) };
                self.stdout.read_exact(self.msgbuf.as_mut_slice())
            })
            .map_err(|io| format!("failed to read from child stdout: {}", io))?;
        let response = bincode::deserialize::<Result<T, String>>(self.msgbuf.as_slice())
            .expect("failed to deserialize message (client)");
        response
    }
}

impl Drop for IpcExternals {
    fn drop(&mut self) {
        if let Err(_) = self.send::<()>(dll::Wow64Message::Stop) {
            // TODO: `log` stuff
            eprintln!("failed to naturally stop wow64 server process, killing");

            // what a beautiful line
            let _ = self.child.kill();
        }
    }
}
