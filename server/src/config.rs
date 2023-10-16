use anyhow::Result;
use std::path::PathBuf;

pub struct PluginConfig {}

impl PluginConfig {
    pub fn from_file(filename: PathBuf) -> Result<PluginConfig> {
        Ok(PluginConfig {})
    }
}
