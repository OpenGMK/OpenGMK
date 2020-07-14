use super::{CallConv, ExternalCall};
use crate::gml;
use shared::{dll, message::MessageStream};
use std::{
    io::{self, Read, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
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
            dll::Value::Str(s) => gml::Value::Str(String::from_utf8_lossy(&s[4..s.len() - 1]).to_string().into()),
        }
    }
}

static mut HELPER: Option<Child> = None;

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

fn make_pipe() -> Pipe<'static> {
    unsafe {
        if HELPER
            .as_mut()
            .and_then(|c| c.try_wait().ok())
            .and_then(|c| if c.is_some() { None } else { Some(()) })
            .is_none()
        {
            HELPER = Some(Command::new("dll-helper.exe").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap());
        }
        Pipe {
            writer: HELPER.as_mut().unwrap().stdin.as_mut().unwrap(),
            reader: HELPER.as_mut().unwrap().stdout.as_mut().unwrap(),
        }
    }
}

pub struct ExternalImpl(u32);

impl ExternalImpl {
    pub fn new(
        dll_name: &str,
        fn_name: &str,
        call_conv: CallConv,
        res_type: dll::ValueType,
        arg_types: &[dll::ValueType],
    ) -> Result<Self, String> {
        let mut pipe = make_pipe();
        pipe.send_message(dll::Message::Define {
            dll_name: dll_name.to_string(),
            fn_name: fn_name.to_string(),
            call_conv,
            res_type,
            arg_types: arg_types.to_vec(),
        })
        .map_err(|e| e.to_string())?;
        pipe.flush().map_err(|e| e.to_string())?;
        // I can't figure out how to extract this into a function without getting lifetime errors
        let mut read_buffer = Vec::new();
        loop {
            match pipe.receive_message::<dll::DefineResult>(&mut read_buffer).map_err(|e| e.to_string())? {
                Some(None) => (),
                Some(Some(message)) => return message.map(ExternalImpl),
                None => return Err("The DLL helper process has been terminated.".into()),
            }
        }
    }
}

impl ExternalCall for ExternalImpl {
    fn call(&self, args: &[gml::Value]) -> Result<gml::Value, String> {
        let mut pipe = make_pipe();
        pipe.send_message(dll::Message::Call { func_id: self.0, args: args.iter().map(|x| x.into()).collect() })
            .map_err(|e| e.to_string())?;
        pipe.flush().map_err(|e| e.to_string())?;
        // I can't figure out how to extract this into a function without getting lifetime errors
        let mut read_buffer = Vec::new();
        loop {
            match pipe.receive_message::<dll::Value>(&mut read_buffer).map_err(|e| e.to_string())? {
                Some(None) => (),
                Some(Some(message)) => return Ok(message.into()),
                None => return Err("The DLL helper process has been terminated.".into()),
            }
        }
    }
}

impl Drop for ExternalImpl {
    fn drop(&mut self) {
        let mut pipe = make_pipe();
        pipe.send_message(dll::Message::Free { func_id: self.0 }).ok();
    }
}
