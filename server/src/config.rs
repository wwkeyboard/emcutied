use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct PluginConfig {
    pub rumqttd_config: PathBuf,
    pub plugins: Vec<Plugin>,
}

#[derive(Deserialize, Debug)]
pub struct Plugin {
    pub name: String,
    pub file: PathBuf,
    pub in_topic: String,
    pub out_topic: Option<String>,
}

impl PluginConfig {
    pub fn from_file(filename: PathBuf) -> Result<PluginConfig> {
        PluginConfig::from_string(std::fs::read_to_string(filename)?)
    }

    pub fn from_string(raw_config: String) -> Result<PluginConfig> {
        let cfg = toml::from_str::<PluginConfig>(&raw_config)?;
        Ok(cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_extracts_base_config_and_plugins() -> Result<()> {
        let pc = PluginConfig::from_string(basic_config())?;

        // base config
        assert_eq!(pc.rumqttd_config, PathBuf::from("./rumqttd.conf"));

        // plugins
        assert_eq!(pc.plugins.len(), 2);
        assert_eq!(pc.plugins.first().unwrap().name, "doubler");
        Ok(())
    }

    #[test]
    fn it_fails_on_bad_config() {
        assert!(PluginConfig::from_string("blahblahblah".to_owned()).is_err());
    }

    fn basic_config() -> String {
        r#"
rumqttd_config = "./rumqttd.conf"

[[plugins]]
name = "doubler"
in_topic = "demo/doubler"
file = "./double_plugin.wasm"

[[plugins]]
name = "another"
in_topic = "demo/another"
file = "./another.wasm"
"#
        .to_owned()
        .trim()
        .to_string()
    }
}
