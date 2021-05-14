use crate::{
    asset::{assert_ver, Error, PascalString, ReadPascalString},
    reader::inflate,
};
use byteorder::{ReadBytesExt, LE};
use std::io::{self, Read, Seek, SeekFrom};

pub const VERSION: u32 = 700;

const ARG_MAX: usize = 17;

pub struct Extension {
    /// The name of the extension.
    pub name: PascalString,

    /// Name of the temporary folder to extract to at runtime.
    pub folder_name: PascalString,

    /// Files contained in the extension.
    pub files: Vec<File>,
}

pub struct File {
    /// The name of the file.
    pub name: PascalString,

    /// What kind of file this is, used to determine the usage.
    pub kind: FileKind,

    /// Initialization code for DLLs or GML files. Not used for other types.
    pub initializer: PascalString,

    /// Finalization code for DLLs or GML files. Not used for other types.
    pub finalizer: PascalString,

    /// GML or external functions you can invoke from the file.
    pub functions: Vec<FileFunction>,

    /// Constants associated with this file.
    pub consts: Vec<FileConst>,

    /// The raw filedata itself.
    pub contents: Box<[u8]>,
}

pub struct FileConst {
    pub name: PascalString,
    pub value: PascalString,
}

/// These const values are in line with the GM8 format. There is no zero.
#[derive(Copy, Clone, PartialEq)]
pub enum FileKind {
    DynamicLibrary = 1,
    GmlScript = 2,
    ActionLibrary = 3,
    Other = 4,
}

impl From<u32> for FileKind {
    fn from(n: u32) -> Self {
        match n {
            x if x == Self::DynamicLibrary as u32 => Self::DynamicLibrary,
            x if x == Self::GmlScript as u32 => Self::GmlScript,
            x if x == Self::ActionLibrary as u32 => Self::ActionLibrary,
            x if x == Self::Other as u32 => Self::Other,
            _ => Self::Other,
        }
    }
}

/// This is in line with GM8 data and there is no value corresponding to zero.
#[derive(Copy, Clone, PartialEq)]
pub enum FunctionValueKind {
    GMString = 1,
    GMReal = 2,
}

impl From<u32> for FunctionValueKind {
    fn from(n: u32) -> Self {
        match n {
            x if x == Self::GMString as u32 => Self::GMString,
            x if x == Self::GMReal as u32 => Self::GMReal,
            _ => Self::GMReal,
        }
    }
}

pub struct FileFunction {
    pub name: PascalString,
    pub external_name: PascalString,
    pub convention: CallingConvention,
    pub id: u32,
    pub arg_count: i32,
    pub arg_types: [FunctionValueKind; ARG_MAX],
    pub return_type: FunctionValueKind,
}

#[derive(Copy, Clone, PartialEq)]
pub enum CallingConvention {
    Gml = 2,
    Stdcall = 11,
    Cdecl = 12,
    Unknown,
}

impl From<u32> for CallingConvention {
    fn from(n: u32) -> Self {
        match n {
            x if x == Self::Gml as u32 => Self::Gml,
            x if x == Self::Stdcall as u32 => Self::Stdcall,
            x if x == Self::Cdecl as u32 => Self::Cdecl,
            _ => Self::Unknown,
        }
    }
}

impl Extension {
    pub fn read(reader: &mut io::Cursor<&mut [u8]>, strict: bool) -> Result<Self, Error> {
        if strict {
            let version = reader.read_u32::<LE>()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let name = reader.read_pas_string()?;
        let folder_name = reader.read_pas_string()?;

        let file_count = reader.read_u32::<LE>()? as usize;
        let mut files: Vec<File> = (0..file_count)
            .map(|_| {
                if strict {
                    let version = reader.read_u32::<LE>()?;
                    assert_ver(version, VERSION)?;
                } else {
                    reader.seek(SeekFrom::Current(4))?;
                }

                let name = reader.read_pas_string()?;
                let kind = FileKind::from(reader.read_u32::<LE>()?);
                let initializer = reader.read_pas_string()?;
                let finalizer = reader.read_pas_string()?;

                let function_count = reader.read_u32::<LE>()? as usize;
                let functions = (0..function_count)
                    .map(|_| {
                        if strict {
                            let version = reader.read_u32::<LE>()?;
                            assert_ver(version, VERSION)?;
                        } else {
                            reader.seek(SeekFrom::Current(4))?;
                        }

                        let name = reader.read_pas_string()?;
                        let external_name = reader.read_pas_string()?;
                        let convention: CallingConvention = reader.read_u32::<LE>()?.into();

                        let id = reader.read_u32::<LE>()?;

                        let arg_count = reader.read_i32::<LE>()?;
                        let mut arg_types = [FunctionValueKind::GMReal; ARG_MAX];
                        for val in arg_types.iter_mut() {
                            *val = reader.read_u32::<LE>()?.into();
                        }
                        let return_type: FunctionValueKind = reader.read_u32::<LE>()?.into();

                        Ok(FileFunction { name, external_name, convention, id, arg_count, arg_types, return_type })
                    })
                    .collect::<Result<_, Error>>()?;

                let const_count = reader.read_u32::<LE>()? as usize;
                let consts = (0..const_count)
                    .map(|_| {
                        if strict {
                            let version = reader.read_u32::<LE>()?;
                            assert_ver(version, VERSION)?;
                        } else {
                            reader.seek(SeekFrom::Current(4))?;
                        }

                        Ok(FileConst { name: reader.read_pas_string()?, value: reader.read_pas_string()? })
                    })
                    .collect::<Result<_, Error>>()?;

                Ok(File { name, kind, initializer, finalizer, functions, consts, contents: Box::new([]) })
            })
            .collect::<Result<_, Error>>()?;

        let contents_len = reader.read_u32::<LE>()? as usize - 4;
        let seed1_raw = reader.read_u32::<LE>()?;
        let data_pos = reader.position() as usize;
        reader.seek(SeekFrom::Current(contents_len as _))?;

        // Don't do decryption if there are no contents
        if contents_len != 0 {
            // decryption setup
            let mut char_table = [0u8; 0x200];
            let mut seed1: i32 = seed1_raw as _;
            let mut seed2: i32 = (seed1 % 0xFA) + 6;
            seed1 /= 0xFA;
            if seed1 < 0 {
                seed1 += 100;
            }
            if seed2 < 0 {
                seed2 += 100;
            }
            for (i, val) in char_table.iter_mut().enumerate() {
                *val = (i % 256) as u8; // 0-255 repeating (twice)
            }

            // calculating char table - pass 1: pseudorandom byteswap
            for i in 1..0x2711 {
                let idx: usize = ((((i * seed2 as u32) + seed1 as u32) % 0xFE) + 1) as _;
                let b1 = char_table[idx];
                let b2 = char_table[idx + 1];
                char_table[idx] = b2;
                char_table[idx + 1] = b1;
            }

            // .. pass 2: use low half to scramble top half
            for i in 0..0x100 {
                let lo: u8 = char_table[i + 1];
                char_table[lo as usize + 0x100] = (i as u8).wrapping_add(1);
            }

            // decrypt data chunk
            for byte in &mut reader.get_mut()[data_pos + 1..data_pos + contents_len] {
                *byte = char_table[*byte as usize + 0x100];
            }

            let end_pos = reader.position() as usize;
            reader.set_position(data_pos as u64);

            // write file chunks
            for file in &mut files {
                if file.kind != FileKind::ActionLibrary {
                    let len = reader.read_u32::<LE>()? as usize;
                    let pos = reader.position() as usize;

                    reader.seek(SeekFrom::Current(len as i64))?; // pre-check for next get
                    let mut file_bytes = Vec::new();
                    inflate(reader.get_ref().get(pos..pos + len).unwrap_or_else(|| unreachable!()))
                        .read_to_end(&mut file_bytes)?;
                    file.contents = file_bytes.into_boxed_slice();
                }
            }

            reader.set_position(end_pos as u64);
        }

        Ok(Extension { name, folder_name, files })
    }
}
