use std::path::PathBuf;

use anyhow::{Context, Result};
use log::info;

const PLUGIN_FUNCTION: &str = "handle";

pub struct Plugin {
    plugin: extism::Plugin<'static>,
    pub in_topic: String,
    pub out_topic: String,
    pub file: PathBuf,
    pub name: String,
}

impl Plugin {
    pub fn new(file: PathBuf, name: &str, in_topic: &str, out_topic: &str) -> Result<Plugin> {
        // load from file
        let plugin = Plugin::load_plugin(&file)?;

        // get links from broker
        let p = Plugin {
            plugin,
            in_topic: in_topic.to_owned(),
            out_topic: out_topic.to_owned(),
            file,
            name: name.to_owned(),
        };
        Ok(p)
    }

    /// load the plugin's code from the `path`
    fn load_plugin(path: &PathBuf) -> Result<extism::Plugin<'static>> {
        info!("loading plugin from path: {path:?}");

        let wasm = std::fs::read(path)
            .with_context(|| format!("reading plugin {} from filesystem", path.display()))?;

        extism::Plugin::create(wasm, [], false)
    }

    pub fn run(&mut self, message: &str) -> Result<Vec<u8>> {
        self.plugin.call(PLUGIN_FUNCTION, message).map(|r| {
            // the result is owned by the plugin, this copies it
            // into new memory
            Vec::from(r)
        })
    }
}
