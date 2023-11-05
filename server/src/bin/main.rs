use anyhow::Result;
use clap::Parser;
use log::{info, warn};
use pretty_env_logger;
use rumqttd::{Broker, Notification};
use server::config::PluginConfig;
use server::plugin::{self, start_plugin, Plugin};
use server::router::{self, Router};
use server::rumqttd::Rumqttd;

use std::{path::PathBuf, thread};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config_file: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    pretty_env_logger::init();

    let main_config = PluginConfig::from_file(args.config_file).expect("parsing config file");

    let mqttd = Rumqttd::new(main_config.rumqttd_config);

    info!("-- create broker link named 'pluginnode'");
    let mut plugin_node = mqttd.link("pluginnode").unwrap();
    plugin_node.link_tx.subscribe("#").unwrap();

    let mut router = Router::new();

    info!("-- start plugins");
    for plugin_config in main_config.plugins {
        let plugin = Plugin::new(
            plugin_config.file,
            plugin_config.in_topic.as_str(),
            plugin_config.out_topic.as_str(),
        )?;

        router.add(Box::new(plugin));
    }

    // Now that the plugins are started this consumes mqttd and starts the server
    mqttd.start();

    router.run(plugin_node.link_rx);

    Ok(())
}
