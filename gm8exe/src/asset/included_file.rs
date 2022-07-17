use crate::{
    asset::{assert_ver, Asset, Error, PascalString, ReadChunk, ReadPascalString, WritePascalString},
    GameVersion,
};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read};

pub const VERSION: u32 = 800;

pub struct IncludedFile {
    /// The name of the included file.
    pub file_name: PascalString,

    /// The path of the source file (from the developer's PC).
    pub source_path: PascalString,

    /// Whether the file data exists.
    pub data_exists: bool,

    /// The length of the source file.
    pub source_length: usize,

    /// Whether the file is embedded.
    pub stored_in_gmk: bool,

    /// Contains the embedded data, if it is embedded.
    pub embedded_data: Option<Box<[u8]>>,

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
    CustomFolder(PascalString),
}

impl Asset for IncludedFile {
    fn deserialize_exe(mut reader: impl Read, _version: GameVersion, strict: bool) -> Result<Self, Error> {
        let version = reader.read_u32::<LE>()?;
        if strict {
            assert_ver(version, VERSION)?;
        }

        let file_name = reader.read_pas_string()?;
        let source_path = reader.read_pas_string()?;

        let data_exists = reader.read_u32::<LE>()? != 0;
        let source_length = reader.read_u32::<LE>()? as usize;
        let stored_in_gmk = reader.read_u32::<LE>()? != 0;

        let embedded_data = if stored_in_gmk && data_exists {
            let len = reader.read_u32::<LE>()? as usize;
            Some(reader.read_chunk(len)?.into_boxed_slice())
        } else {
            None
        };

        let export_flag = reader.read_u32::<LE>()?;
        let custom_folder_path = reader.read_pas_string()?; // always present
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
            file_name,
            source_path,
            data_exists,
            source_length,
            stored_in_gmk,
            embedded_data,
            export_settings,
            overwrite_file,
            free_memory,
            remove_at_end,
        })
    }

    fn serialize_exe(&self, mut writer: impl io::Write, _version: GameVersion) -> io::Result<()> {
        writer.write_u32::<LE>(VERSION)?;
        writer.write_pas_string(&self.file_name)?;
        writer.write_pas_string(&self.source_path)?;
        writer.write_u32::<LE>(self.data_exists.into())?;
        writer.write_u32::<LE>(self.source_length as u32)?;
        writer.write_u32::<LE>(self.stored_in_gmk.into())?;
        if let Some(data) = &self.embedded_data {
            writer.write_u32::<LE>(data.len() as u32)?;
            // TODO: How many years old is this comment below?
            // I imagine the data's length is meant to be written?
            // It's not like anybody is testing serialisation.
            // TODO: minio.write_buffer?
            writer.write_all(data)?;
            //data.len();
        }
        match &self.export_settings {
            ExportSetting::NoExport => {
                writer.write_u32::<LE>(0)?;
                writer.write_pas_string(&"".into())?;
            },
            ExportSetting::TempFolder => {
                writer.write_u32::<LE>(1)?;
                writer.write_pas_string(&"".into())?;
            },
            ExportSetting::GameFolder => {
                writer.write_u32::<LE>(2)?;
                writer.write_pas_string(&"".into())?;
            },
            ExportSetting::CustomFolder(folder) => {
                writer.write_u32::<LE>(3)?;
                writer.write_pas_string(folder)?;
            },
        }
        writer.write_u32::<LE>(self.overwrite_file.into())?;
        writer.write_u32::<LE>(self.free_memory.into())?;
        writer.write_u32::<LE>(self.remove_at_end.into())?;
        Ok(())
    }
}
