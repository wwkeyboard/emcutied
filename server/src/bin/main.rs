use anyhow::Result;
use clap::Parser;
use log::{info, warn};
use pretty_env_logger;
use rumqttd::{Broker, Notification};
use server::config::PluginConfig;
use server::plugin::{self, start_plugin, Plugin};
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

    // monitornode is what prints to stdout
    let mut monitor = mqttd.link("monitornode")?;

    info!("-- monitornode subscribe to #");
    monitor.link_tx.subscribe("#").unwrap();

    info!("-- create broker link named 'pluginnode_sender'");
    let mut sender = mqttd.link("pluginnode_sender").unwrap();

    info!("-- create broker link named 'pluginnode'");
    let mut plugin_node = mqttd.link("pluginnode").unwrap();
    plugin_node.link_tx.subscribe("#").unwrap();

    info!("-- start plugins");
    //    if let Some(plugin_filename) = args.plugin_file {
    for plugin_config in main_config.plugins {
        //let plugin = plugin::load_plugin(plugin_filename, sender.link_tx)?;
        let plugin = Plugin::new(
            plugin_config.file,
            &mqttd,
            plugin_config.in_topic.as_str(),
            plugin_config.out_topic.as_str(),
        )?;

        tokio::spawn(async move {
            plugin.run().await;
        });
    }

    // Now that the plugins are started this consumes mqttd and starts the server
    mqttd.start();

    let mut count = 0;
    loop {
        let notification = match monitor.link_rx.recv().unwrap() {
            Some(v) => v,
            None => continue,
        };

        match notification {
            Notification::Forward(forward) => {
                count += 1;
                println!(
                    "Topic = {:?}, Count = {}, Payload = {} bytes",
                    forward.publish.topic,
                    count,
                    forward.publish.payload.len()
                );
            }
            v => {
                warn!("unknown message {v:?}");
            }
        }
    }
}
