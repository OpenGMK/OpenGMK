use crate::{
    asset::{assert_ver, Asset, AssetDataError, PascalString, ReadPascalString, WritePascalString},
    GameVersion,
};

use minio::{ReadPrimitives, WritePrimitives};
use std::io::{self, Seek, SeekFrom};

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
    fn deserialize<B>(bytes: B, strict: bool, _version: GameVersion) -> Result<Self, AssetDataError>
    where
        B: AsRef<[u8]>,
        Self: Sized,
    {
        let mut reader = io::Cursor::new(bytes.as_ref());

        if strict {
            let version = reader.read_u32_le()?;
            assert_ver(version, VERSION)?;
        } else {
            reader.seek(SeekFrom::Current(4))?;
        }

        let file_name = reader.read_pas_string()?;
        let source_path = reader.read_pas_string()?;

        let data_exists = reader.read_u32_le()? != 0;
        let source_length = reader.read_u32_le()? as usize;
        let stored_in_gmk = reader.read_u32_le()? != 0;

        let embedded_data = if stored_in_gmk && data_exists {
            // TODO: this should be minio::read_buffer? Does that function exist yet?
            let len = reader.read_u32_le()? as usize;
            let pos = reader.position() as usize;
            reader.seek(SeekFrom::Current(len as i64))?;
            Some(reader.get_ref().get(pos..pos + len).unwrap_or_else(|| unreachable!()).to_vec().into_boxed_slice())
        } else {
            None
        };

        let export_flag = reader.read_u32_le()?;
        let custom_folder_path = reader.read_pas_string()?; // always present
        let export_settings = match export_flag {
            0 => ExportSetting::NoExport,
            1 => ExportSetting::TempFolder,
            2 => ExportSetting::GameFolder,
            _ => ExportSetting::CustomFolder(custom_folder_path),
        };

        let overwrite_file = reader.read_u32_le()? != 0;
        let free_memory = reader.read_u32_le()? != 0;
        let remove_at_end = reader.read_u32_le()? != 0;

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

    fn serialize<W>(&self, writer: &mut W) -> io::Result<usize>
    where
        W: io::Write,
    {
        let mut result = writer.write_u32_le(VERSION)?;
        result += writer.write_pas_string(&self.file_name)?;
        result += writer.write_pas_string(&self.source_path)?;
        result += writer.write_u32_le(self.data_exists as u32)?;
        result += writer.write_u32_le(self.source_length as u32)?;
        result += writer.write_u32_le(self.stored_in_gmk as u32)?;
        if let Some(data) = &self.embedded_data {
            result += writer.write_u32_le(data.len() as u32)?;
            // TODO: minio.write_buffer?
            writer.write_all(data)?;
            result += data.len();
        }
        match &self.export_settings {
            ExportSetting::NoExport => {
                result += writer.write_u32_le(0)?;
                result += writer.write_pas_string(&"".into())?;
            },
            ExportSetting::TempFolder => {
                result += writer.write_u32_le(1)?;
                result += writer.write_pas_string(&"".into())?;
            },
            ExportSetting::GameFolder => {
                result += writer.write_u32_le(2)?;
                result += writer.write_pas_string(&"".into())?;
            },
            ExportSetting::CustomFolder(folder) => {
                result += writer.write_u32_le(3)?;
                result += writer.write_pas_string(folder)?;
            },
        }
        result += writer.write_u32_le(self.overwrite_file as u32)?;
        result += writer.write_u32_le(self.free_memory as u32)?;
        result += writer.write_u32_le(self.remove_at_end as u32)?;
        Ok(result)
    }
}
