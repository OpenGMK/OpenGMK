#![cfg(not(all(target_os = "windows", target_arch = "x86")))]

use super::dll;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use dll::PROTOCOL_VERSION;
use serde::de;
use std::{
    env,
    io::{Read, Write},
    process,
};

const PROCESS_DEFAULT_NAME: &str = "gm8emulator-wow64.exe";
const PROCESS_ENV_OVERRIDE: &str = "OPENGMK_WOW64_BINARY";

pub struct IpcExternal(i32);

struct ChildProcess {
    pub child: process::Child,
    msgbuf: Vec<u8>,
    pub stdin: process::ChildStdin,
    pub stdout: process::ChildStdout,
}

pub struct IpcManager {
    child: Option<ChildProcess>,
}

impl IpcManager {
    pub fn new() -> Self {
        Self { child: None }
    }

    pub fn define(&mut self, signature: &dll::ExternalSignature) -> Result<IpcExternal, String> {
        // would just use get_or_insert for this but then i can't throw errors
        let child = match &mut self.child {
            Some(child) => child,
            None => self.child.get_or_insert(ChildProcess::new()?),
        };
        child.send(dll::Wow64Message::Define(signature.clone())).map(IpcExternal)
    }

    pub fn call(&mut self, external: &IpcExternal, args: &[dll::Value]) -> dll::Value {
        self.child.as_mut().unwrap().send(dll::Wow64Message::Call(external.0, args.to_vec())).unwrap()
    }

    pub fn free(&mut self, external: IpcExternal) {
        // eh who cares, don't error here
        self.child.as_mut().and_then(|c| c.send::<()>(dll::Wow64Message::Free(external.0)).ok());
    }
}

impl ChildProcess {
    pub fn new() -> Result<Self, String> {
        let mut process_path = env::current_exe().expect("failed to query path to current executable");
        process_path.set_file_name(match env::var_os(PROCESS_ENV_OVERRIDE) {
            Some(name) => name,
            None => PROCESS_DEFAULT_NAME.into(),
        });
        if !process_path.exists() {
            return Err(format!(
                "{} was not found, please extract it into your gm8emulator directory",
                PROCESS_DEFAULT_NAME
            ))
        }
        let mut child = process::Command::new(process_path)
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .arg(dll::PROTOCOL_VERSION.to_string())
            .spawn()
            .map_err(|e| format!("failed to spawn child process: {}", e))?;
        let stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        // make sure we're talking to the right version
        let mut version = [0; 2];
        stdout.read_exact(&mut version).map_err(|e| format!("couldn't receive gm8emulator-wow64 version: {}", e))?;
        let version = u16::from_le_bytes(version);
        if version != dll::PROTOCOL_VERSION {
            return Err(format!("gm8emulator-wow64 version mismatch (expected {}, got {})", PROTOCOL_VERSION, version))
        }
        Ok(Self { child, msgbuf: Vec::new(), stdin, stdout })
    }

    pub fn send<T>(&mut self, message: dll::Wow64Message) -> Result<T, String>
    where
        T: for<'de> de::Deserialize<'de>,
    {
        self.msgbuf.clear();
        bincode::serialize_into(&mut self.msgbuf, &message).expect("failed to serialize message (client)");
        assert!(self.msgbuf.len() <= u32::MAX as usize);
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

impl Drop for ChildProcess {
    fn drop(&mut self) {
        if let Err(_) = self.send::<()>(dll::Wow64Message::Stop) {
            // TODO: `log` stuff
            eprintln!("failed to naturally stop wow64 server process, killing");

            // what a beautiful line
            let _ = self.child.kill();
        }
    }
}
