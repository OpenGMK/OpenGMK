use byteorder::{LE, ReadBytesExt};
use crate::asset::{ByteString, Version};
use std::io::{self, Read};

const ARG_MAX: usize = 17;

pub struct Extension {
    pub name: ByteString,
    pub version: Version,

    pub folder_name: ByteString,
    pub files: Vec<File>,
}

// Section: Child structs

pub struct File {
    pub name: ByteString,
    pub version: Version,

    pub kind: FileKind,
    pub initializer: ByteString,
    pub finalizer: ByteString,
    pub functions: Vec<FileFunction>,
    pub consts: Vec<FileConst>,
    pub contents: Box<[u8]>,
}

pub struct FileConst {
    pub name: ByteString,
    pub version: Version,
    pub value: ByteString,
}

/// These const values are in line with the GM8 format. There is no zero.
#[derive(Copy, Clone, PartialEq)]
pub enum FileKind {
    DynamicLibrary = 1,
    GmlScript = 2,
    ActionLibrary = 3,
    Other = 4,
}

/// This is in line with GM8 data and there is no value corresponding to zero.
#[derive(Copy, Clone, PartialEq)]
pub enum FunctionValueKind {
    GMString = 1,
    GMReal = 2,
}

pub struct FileFunction {
    pub name: ByteString,
    pub version: Version,
    pub external_name: ByteString,
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

// Section: implementations of From<T>

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

impl From<u32> for FunctionValueKind {
    fn from(n: u32) -> Self {
        match n {
            x if x == Self::GMString as u32 => Self::GMString,
            x if x == Self::GMReal as u32 => Self::GMReal,
            _ => Self::GMReal,
        }
    }
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

// Section: Extension read/write

impl Extension {
    pub(crate) fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let version = read_version!(reader, "{unknown}", is_gmk, "script", Gm700)?;
        let name = ByteString::read(&mut reader)?;
        let folder_name = ByteString::read(&mut reader)?;

        let file_count = reader.read_u32::<LE>()? as usize;
        let mut files: Vec<File> = (0..file_count)
            .map(|_| {
                let version = read_version!(reader, name, is_gmk, "script", Gm700)?;
                let name = ByteString::read(&mut reader)?;
                let kind = FileKind::from(reader.read_u32::<LE>()?);
                let initializer = ByteString::read(&mut reader)?;
                let finalizer = ByteString::read(&mut reader)?;

                let function_count = reader.read_u32::<LE>()? as usize;
                let functions = (0..function_count)
                    .map(|_| {
                        let version = read_version!(reader, name, is_gmk, "script", Gm700)?;
                        let name = ByteString::read(&mut reader)?;
                        let external_name = ByteString::read(&mut reader)?;
                        let convention: CallingConvention = reader.read_u32::<LE>()?.into();

                        let id = reader.read_u32::<LE>()?;

                        let arg_count = reader.read_i32::<LE>()?;
                        let mut arg_types = [FunctionValueKind::GMReal; ARG_MAX];
                        for val in arg_types.iter_mut() {
                            *val = reader.read_u32::<LE>()?.into();
                        }
                        let return_type: FunctionValueKind = reader.read_u32::<LE>()?.into();

                        Ok(FileFunction { name, version, external_name, convention, id, arg_count, arg_types, return_type })
                    })
                    .collect::<io::Result<_>>()?;

                let const_count = reader.read_u32::<LE>()? as usize;
                let consts = (0..const_count)
                    .map(|_| {
                        let version = read_version!(reader, name, is_gmk, "script", Gm700)?;
                        Ok(FileConst { name: ByteString::read(&mut reader)?, version, value: ByteString::read(&mut reader)? })
                    })
                    .collect::<io::Result<_>>()?;

                Ok(File { name, version, kind, initializer, finalizer, functions, consts, contents: Box::new([]) })
            })
            .collect::<io::Result<_>>()?;

        let contents_len = reader.read_u32::<LE>()? - 4;
        let mut seed1 = reader.read_i32::<LE>()?;
        let mut seed2 = (seed1 % 0xFA) + 6;

        // Don't do decryption if there are no contents
        if contents_len != 0 {
            // decryption setup
            let mut char_table = [0u8; 0x100];
            let mut reverse_table = [0u8; 0x100];
            seed1 /= 0xFA;
            if seed1 < 0 {
                seed1 += 100;
            }
            if seed2 < 0 {
                seed2 += 100;
            }
            for (i, val) in char_table.iter_mut().enumerate() {
                *val = i as u8; // 0-255
            }

            // calculating char table - pass 1: pseudorandom byteswap
            for i in 1..0x2711 {
                let idx = ((((i * seed2 as u32) + seed1 as u32) % 0xFE) + 1) as usize;
                char_table.swap(idx, idx + 1);
            }

            // calculating reverse table - pass 2: use char table to generate reverse table
            for (i, b) in char_table.iter().copied().enumerate().skip(1) {
                unsafe {
                    // SAFETY: reverse_table is an array of 256 elements; the highest possible index here is u8::MAX
                    *reverse_table.get_unchecked_mut(usize::from(b)) = i as u8;
                }
            }
            unsafe {
                // SAFETY: as above
                let b = usize::from(reverse_table[0]);
                *reverse_table.get_unchecked_mut(b) = 0;
            }

            // Decrypt, decompress and write file chunks
            let mut data_consumed = 0u32;
            let mut first_byte = Some(reader.read_u8()?); // First byte is not encrypted
            for file in &mut files {
                if file.kind != FileKind::ActionLibrary {
                    // SAFETY: reverse_table is an array of 256 elements; the highest possible index here is u8::MAX = 255
                    let len = match first_byte.take() {
                        Some(b) => unsafe {
                            u32::from_le_bytes([
                                b,
                                *reverse_table.get_unchecked(usize::from(reader.read_u8()?)),
                                *reverse_table.get_unchecked(usize::from(reader.read_u8()?)),
                                *reverse_table.get_unchecked(usize::from(reader.read_u8()?)),
                            ])
                        },
                        None => unsafe {
                            u32::from_le_bytes([
                                *reverse_table.get_unchecked(usize::from(reader.read_u8()?)),
                                *reverse_table.get_unchecked(usize::from(reader.read_u8()?)),
                                *reverse_table.get_unchecked(usize::from(reader.read_u8()?)),
                                *reverse_table.get_unchecked(usize::from(reader.read_u8()?)),
                            ])
                        },
                    };

                    data_consumed += len + 4;
                    if data_consumed > contents_len {
                        return Err(io::Error::from(io::ErrorKind::InvalidData));
                    }

                    let mut contents = Vec::new();
                    flate2::read::ZlibDecoder::new(SwapReader::new(reader, len as _, &reverse_table)).read_to_end(&mut contents)?;
                    file.contents = contents.into();
                }
            }
        }

        Ok(Extension { name, version, folder_name, files })
    }
}

struct SwapReader<'a> {
    reader: &'a mut dyn Read,
    len: usize,
    swap: &'a [u8; 0x100],
}

impl<'a> SwapReader<'a> {
    fn new(reader: &'a mut dyn Read, len: usize, swap: &'a [u8; 0x100]) -> Self {
        Self { reader, len, swap }
    }
}

impl<'a> Read for SwapReader<'a> {
    fn read(&mut self, t: &mut [u8]) -> io::Result<usize> {
        // SAFETY: `swap` is an array of 256 elements; the highest possible index here is u8::MAX
        let count = if let Some(t) = t.get_mut(..self.len) { self.reader.read(t)? } else { self.reader.read(t)? };
        t.iter_mut().take(count).for_each(|b| *b = unsafe { *self.swap.get_unchecked(usize::from(*b)) });
        self.len -= count;
        Ok(count)
    }
}

impl<'a> Drop for SwapReader<'a> {
    fn drop(&mut self) {
        if self.len != 0 {
            // We didn't consume all the bytes that were supposed to be decompressed.
            // This should never happen, so one allocation here is fine.
            let mut v = Vec::with_capacity(self.len);
            unsafe { v.set_len(self.len); }
            let _ = self.reader.read_exact(&mut v);
        }
    }
}
