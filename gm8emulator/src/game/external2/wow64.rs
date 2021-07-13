use std::{env, process};
use super::{dll, ID};

const PROCESS_DEFAULT_NAME: &str = "gm8emulator-wow64.exe";
const PROCESS_ENV_OVERRIDE: &str = "OPENGMK_WOW64_BINARY";

pub struct IpcExternals {
    process: process::Child,
}

impl IpcExternals {
    pub fn new() -> Result<Self, String> {
        let mut process_path = env::current_exe()
            .expect("failed to query path to current executable");
        process_path.set_file_name(match env::var_os(PROCESS_DEFAULT_NAME) {
            Some(name) => name,
            None => PROCESS_DEFAULT_NAME.into(),
        });
        let process = process::Command::new(process_path)
            .stdin(process::Stdio::piped())
            .stdout(process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn child process: {}", e))?;

        todo!()
    }

    pub fn define(
        &mut self,
        dll: &str,
        symbol: &str,
        call_conv: dll::CallConv,
        type_args: &[dll::ValueType],
        type_return: dll::ValueType,
    ) -> Result<ID, String> {
        todo!()
    }

    pub fn free(&mut self, id: ID) -> Result<(), String> {
        todo!()
    }
}
