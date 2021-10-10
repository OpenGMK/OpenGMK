#[cfg(not(all(target_os = "windows", target_arch = "x86")))]
compile_error!("this crate cannot be built for a target other than windows i686");

#[path = "../../gm8emulator/src/game/external/dll.rs"]
mod dll;
#[path = "../../gm8emulator/src/game/external/win32.rs"]
mod win32;
#[path = "../../gm8emulator/src/handleman.rs"]
#[allow(dead_code)]
mod handleman;

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use handleman::{HandleList, HandleManager};
use std::{env, io::{self, Read, Write}, time::{Duration}};

struct Manager {
    externals: HandleList<win32::NativeExternal>,
    manager: win32::NativeManager,
}

impl Manager {
    fn define(&mut self, signature: dll::ExternalSignature) -> Result<i32, String> {
        Ok(self.externals.put(self.manager.define(&signature)?))
    }

    fn call(&mut self, id: i32, args: &[dll::Value]) -> Result<dll::Value, String> {
        Ok(self.manager.call(self.externals.get(id).unwrap(), args))
    }

    fn free(&mut self, id: i32) -> Result<(), String> {
        self.externals.delete(id);
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut manager = Manager { externals: HandleList::new(), manager: win32::NativeManager::new() };
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // get hello before anything else
    // if it takes over half a second, my dude actually just clicked the exe
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut version = [0; 2];
        io::stdin().read_exact(&mut version).unwrap();
        tx.send(u16::from_le_bytes(version)).unwrap();
    });
    let version = match rx.recv_timeout(Duration::from_millis(500)) {
        Ok(v) => v,
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return Ok(()), // reading errored
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            eprintln!("This is a bridge executable, and is not meant to be ran independently.");
            eprintln!("<< Press Return >>");
            stdin.lock(); // pressing return will send 2 bytes to stdin, which unlocks the other thread
            return Ok(())
        }
    };
    // send the version back anyway, if it's wrong then no need to stick around
    stdout.write_all(&dll::PROTOCOL_VERSION.to_le_bytes())?;
    stdout.flush()?;
    if version != dll::PROTOCOL_VERSION {
        return Ok(())
    }

    eprintln!("starting dll compatibility layer\n  > server: \"{}\"", env::args().next().unwrap());

    let mut message = Vec::with_capacity(1024);
    loop {
        message.clear();

        let length = stdin.read_u32::<LE>()? as usize;
        unsafe { message.set_len(length) };
        stdin.read_exact(message.as_mut_slice())?;

        macro_rules! respond {
            ($res:expr) => {{
                let result = $res;
                message.clear();
                bincode::serialize_into(&mut message, &result).expect("failed to serialize message (server)");
                assert!(message.len() <= u32::max_value() as usize);
                stdout.write_u32::<LE>(message.len() as u32)?;
                stdout.write_all(&message[..])?;
                stdout.flush()?;
            }};
        }

        match bincode::deserialize::<dll::Wow64Message>(&message)
            .expect("failed to deserialize message (server)")
        {
            dll::Wow64Message::Call(id, args)
                => respond!(manager.call(id, &args)),
            dll::Wow64Message::Define(signature)
                => respond!(manager.define(signature)),
            dll::Wow64Message::Free(id)
                => respond!(manager.free(id)),
            dll::Wow64Message::Stop => {
                respond!(Result::<(), String>::Ok(()));
                break Ok(())
            },
        }
    }
}
