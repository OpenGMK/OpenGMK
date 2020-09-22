use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IncludedFile {
    pub name: String,
    pub data: Option<Box<[u8]>>,
    pub export_settings: ExportSetting,
    pub overwrite: bool,
    pub free_after_export: bool,
    pub remove_at_end: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ExportSetting {
    NoExport,
    TempFolder,
    GameFolder,
    CustomFolder(String),
}

impl IncludedFile {
    pub fn export(&mut self, temp_directory: PathBuf, program_directory: PathBuf) -> std::io::Result<()> {
        if self.data.is_some() {
            if let Some(mut export_path) = match self.export_settings.clone() {
                ExportSetting::NoExport => None,
                ExportSetting::TempFolder => Some(temp_directory),
                ExportSetting::GameFolder => Some(program_directory),
                ExportSetting::CustomFolder(dir) => Some(dir.clone().into()),
            } {
                export_path.push(&self.name);
                self.export_to(&export_path)?;
            }
        }
        Ok(())
    }

    pub fn export_to(&mut self, path: &Path) -> std::io::Result<()> {
        if let Some(data) = self.data.as_ref() {
            if self.overwrite || !path.exists() {
                std::fs::write(path, &data)?;
            }
            if self.free_after_export {
                self.data = None;
            }
        }
        Ok(())
    }
}
