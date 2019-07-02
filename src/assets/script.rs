use crate::bytes::{ReadBytes, ReadString, WriteBytes, WriteString};
use crate::game::parser::ParserOptions;
use crate::gml::ast::{self, AST};
use crate::types::Version;
use std::io::{self, Seek, SeekFrom};
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::NonNull;

pub const VERSION: Version = 800;

pub struct Script<'a> {
    /// The asset name present in GML and the editor.
    pub name: String,

    /// The full source code for the script.
    pub source: Box<str>,

    /// AST for the script's source code.
    pub ast: Result<AST<'a>, ast::Error>,

    // Do not implement Unpin!
    _no_unpin: PhantomPinned,
}

impl<'a> Script<'a> {
    pub fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_pas_string(&self.name)?;
        result += writer.write_u32_le(VERSION as u32)?;
        result += writer.write_pas_string(&self.source)?;

        Ok(result)
    }

    pub fn deserialize<B>(bin: B, options: &ParserOptions) -> io::Result<Pin<Box<Script<'a>>>>
    where
        B: AsRef<[u8]>,
    {
        let mut reader = io::Cursor::new(bin.as_ref());
        let name = reader.read_pas_string()?;

        if options.strict {
            let version = reader.read_u32_le()?;
            assert_eq!(version, VERSION);
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let source = reader.read_pas_string()?.into_boxed_str();
        let mut script = Box::pin(Script {
            name,
            source,
            ast: Ok(AST::empty()),

            _no_unpin: PhantomPinned,
        });

        // Since modifying a field will not move it, this is safe.
        // This is intended Pin usage. https://doc.rust-lang.org/std/pin/index.html
        let source_ptr = NonNull::from(&script.source);
        unsafe {
            let mut_ref: Pin<&mut Self> = Pin::as_mut(&mut script);
            Pin::get_unchecked_mut(mut_ref).ast = AST::new(&*source_ptr.as_ptr());
        }

        Ok(script)
    }
}
