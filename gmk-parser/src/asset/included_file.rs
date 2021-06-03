use crate::asset::{Asset, ByteString, Timestamp, Version};

use byteorder::{LE, ReadBytesExt, WriteBytesExt};
use std::io;

pub struct IncludedFile {
    // pub name: ByteString,
    // See `file_name` for name.
    pub timestamp: Timestamp,
    pub version: Version,

    /// The name of the included file.
    pub file_name: ByteString,

    /// The path of the source file (from the developer's PC).
    pub source_path: ByteString,

    /// The length of the source file.
    pub source_length: u32,

    /// Whether the file is embedded.
    pub stored_in_gmk: bool,

    /// Contains the embedded data, if it is embedded.
    pub embedded_data: Option<Vec<u8>>,

    /// The export settings used for the file on load.
    pub export_settings: ExportSetting,

    /// Overwrite file if it exists (while exporting).
    pub overwrite_file: bool,

    /// Whether to free memory after exporting.
    /// Why is this an option.
    pub free_memory: bool,

    /// Whether to delete the exported external file at the end.
    pub remove_at_end: bool,
}

pub enum ExportSetting {
    NoExport,
    TempFolder,
    GameFolder,
    CustomFolder(ByteString),
}

impl Asset for IncludedFile {
    #[inline]
    fn name(&self) -> &[u8] {
        self.file_name.0.as_slice()
    }

    #[inline]
    fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    #[inline]
    fn version(&self) -> Version {
        self.version
    }

    fn from_gmk<R: io::Read>(&self, mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, true)
    }

    fn to_gmk<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, true)
    }

    fn from_exe<R: io::Read>(&self, mut reader: R) -> io::Result<Self> {
        Self::read(&mut reader, false)
    }

    fn to_exe<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.write(&mut writer, false)
    }
}

impl IncludedFile {
    fn read(mut reader: &mut dyn io::Read, is_gmk: bool) -> io::Result<Self> {
        let name = ByteString::read(&mut reader)?;
        let timestamp = if is_gmk {
            Timestamp(reader.read_f64::<LE>()?)
        } else {
            Timestamp::default()
        };
        let version = read_version!(reader, name, is_gmk, "included file", Gm800)?;

        let file_name = ByteString::read(&mut reader)?;
        let source_path = ByteString::read(&mut reader)?;
        let embedded_data_exists = reader.read_u32::<LE>()? != 0;
        let source_length = reader.read_u32::<LE>()?;
        let stored_in_gmk = reader.read_u32::<LE>()? != 0;

        let embedded_data = if stored_in_gmk && embedded_data_exists {
            let data_len = reader.read_u32::<LE>()? as usize;
            let mut data = Vec::with_capacity(data_len);
            unsafe { data.set_len(data_len) };
            reader.read_exact(data.as_mut_slice())?;
            Some(data)
        } else {
            None
        };

        let export_flag = reader.read_u32::<LE>()?;
        let custom_folder_path = ByteString::read(&mut reader)?;
        let export_settings = match export_flag {
            0 => ExportSetting::NoExport,
            1 => ExportSetting::TempFolder,
            2 => ExportSetting::GameFolder,
            _ => ExportSetting::CustomFolder(custom_folder_path),
        };
        let overwrite_file = reader.read_u32::<LE>()? != 0;
        let free_memory = reader.read_u32::<LE>()? != 0;
        let remove_at_end = reader.read_u32::<LE>()? != 0;

        Ok(IncludedFile {
            version, timestamp,
            file_name, source_path, source_length,
            stored_in_gmk, embedded_data,
            export_settings, overwrite_file, free_memory, remove_at_end,
        })
    }

    fn write(&self, mut writer: &mut dyn io::Write, is_gmk: bool) -> io::Result<()> {
        assert_eq!(self.version, Version::Gm800);
        if is_gmk {
            writer.write_f64::<LE>(self.timestamp.0)?;
        }
        writer.write_u32::<LE>(self.version as u32)?;

        self.file_name.write(&mut writer)?;
        self.source_path.write(&mut writer)?;
        writer.write_u32::<LE>(self.embedded_data.is_some().into())?;
        writer.write_u32::<LE>(self.source_length)?;
        writer.write_u32::<LE>(self.stored_in_gmk.into())?;

        // TODO: the logic isn't 100% here (or the other places)
        if self.stored_in_gmk {
            if let Some(data) = &self.embedded_data {
                assert!(data.len() <= u32::max_value() as usize);
                writer.write_u32::<LE>(data.len() as u32)?;
                writer.write_all(data.as_slice())?;
            }
        }

        match &self.export_settings {
            ExportSetting::NoExport => {
                writer.write_u32::<LE>(0)?; // ""
                writer.write_u32::<LE>(0)?;
            },
            ExportSetting::TempFolder => {
                writer.write_u32::<LE>(0)?; // ""
                writer.write_u32::<LE>(1)?;
            },
            ExportSetting::GameFolder => {
                writer.write_u32::<LE>(0)?; // ""
                writer.write_u32::<LE>(2)?;
            },
            ExportSetting::CustomFolder(folder) => {
                folder.write(&mut writer)?;
                writer.write_u32::<LE>(3)?;
            },
        }

        writer.write_u32::<LE>(self.overwrite_file.into())?;
        writer.write_u32::<LE>(self.free_memory.into())?;
        writer.write_u32::<LE>(self.remove_at_end.into())?;
        Ok(())
    }
}
