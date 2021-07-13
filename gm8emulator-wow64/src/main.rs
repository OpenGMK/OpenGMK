#[cfg(not(all(target_os = "windows", target_arch = "x86")))]
compile_error!("this crate cannot be built for a target other than windows i686");

type ID = i32;
#[path = "../../gm8emulator/src/game/external2/dll.rs"]
mod dll;
#[path = "../../gm8emulator/src/game/external2/win32.rs"]
mod win32;

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    let mut externals = win32::NativeExternals::new().unwrap();
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    eprintln!("wow64> compatibility layer started!");
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
                => respond!(externals.call(id, &args)),
            dll::Wow64Message::Define(dll, sym, cconv, args, ret)
                => respond!(externals.define(&dll, &sym, cconv, &args, ret)),
            dll::Wow64Message::DefineDummy(dll, sym, dummy, argc)
                => respond!(externals.define_dummy(&dll, &sym, dummy, argc)),
            dll::Wow64Message::Free(dll)
                => respond!(externals.free(&dll)),
            dll::Wow64Message::Stop => {
                respond!(Result::<(), String>::Ok(()));
                break Ok(())
            },
        }
    }
}
