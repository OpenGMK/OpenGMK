use super::{DefineInfo, ExternalCall};
use crate::gml;
use encoding_rs::Encoding;
use shared::{dll, message::MessageStream};
use std::{
    io::{self, Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    thread,
};

impl From<&gml::Value> for dll::Value {
    fn from(v: &gml::Value) -> Self {
        match v {
            gml::Value::Real(x) => dll::Value::Real(x.into_inner()),
            gml::Value::Str(s) => s.as_ref().into(),
        }
    }
}

impl From<dll::Value> for gml::Value {
    fn from(v: dll::Value) -> Self {
        match v {
            dll::Value::Real(x) => x.into(),
            dll::Value::Str(s) => gml::Value::Str((&s[4..s.len() - 1]).into()),
        }
    }
}

static mut BRIDGE: Option<Child> = None;

struct Pipe<'a> {
    writer: &'a mut ChildStdin,
    reader: &'a mut ChildStdout,
}

impl Read for Pipe<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl Write for Pipe<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

fn make_pipe() -> Result<Pipe<'static>, String> {
    unsafe {
        if BRIDGE
            .as_mut()
            .and_then(|c| c.try_wait().ok())
            .and_then(|c| if c.is_some() { None } else { Some(()) })
            .is_some()
        {
            Ok(Pipe {
                writer: BRIDGE.as_mut().unwrap().stdin.as_mut().unwrap(),
                reader: BRIDGE.as_mut().unwrap().stdout.as_mut().unwrap(),
            })
        } else {
            Err("The bridge process was terminated before it could be invoked.".into())
        }
    }
}

pub struct ExternalImpl(u32);

impl ExternalImpl {
    pub fn new(info: &DefineInfo, encoding: &'static Encoding) -> Result<Self, String> {
        unsafe {
            if BRIDGE.is_none() {
                let mut bridge_path = std::env::current_exe().unwrap();
                bridge_path.set_file_name("dll-bridge.exe");
                assert!(bridge_path.is_file(), "dll-bridge.exe could not be found.");
                BRIDGE = Some(
                    Command::new(bridge_path)
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .map_err(|e| format!("Could not start dll-bridge.exe: {}", e))?,
                );
            }
        }
        let mut pipe = make_pipe()?;
        pipe.send_message(dll::Message::Define {
            dll_name: info.dll_name.decode(encoding).into_owned(),
            fn_name: info.fn_name.decode(encoding).into_owned(),
            call_conv: info.call_conv,
            res_type: info.res_type,
            arg_types: info.arg_types.clone(),
        })
        .map_err(|e| e.to_string())?;
        pipe.flush().map_err(|e| e.to_string())?;
        // I can't figure out how to extract this into a function without getting lifetime errors
        let mut read_buffer = Vec::new();
        loop {
            match pipe.receive_message::<dll::DefineResult>(&mut read_buffer).map_err(|e| e.to_string())? {
                Some(None) => thread::yield_now(),
                Some(Some(message)) => return message.map(ExternalImpl),
                None => return Err("The DLL bridge process was terminated mid-call.".into()),
            }
        }
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, args: &[gml::Value]) -> Result<gml::Value, String> {
        let mut pipe = make_pipe()?;
        pipe.send_message(dll::Message::Call { func_id: self.0, args: args.iter().map(|x| x.into()).collect() })
            .map_err(|e| e.to_string())?;
        pipe.flush().map_err(|e| e.to_string())?;
        // I can't figure out how to extract this into a function without getting lifetime errors
        let mut read_buffer = Vec::new();
        loop {
            match pipe.receive_message::<dll::Value>(&mut read_buffer).map_err(|e| e.to_string())? {
                Some(None) => thread::yield_now(),
                Some(Some(message)) => return Ok(message.into()),
                None => return Err("The DLL bridge process was terminated mid-call.".into()),
            }
        }
    }
}

impl Drop for ExternalImpl {
    fn drop(&mut self) {
        if let Ok(mut pipe) = make_pipe() {
            pipe.send_message(dll::Message::Free { func_id: self.0 }).ok();
        }
    }
}
