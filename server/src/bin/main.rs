use anyhow::Result;
use clap::Parser;
use log::{info, warn};
use pretty_env_logger;
use rumqttd::{Broker, Notification};
use server::plugin::{self, start_plugin};

use std::{path::PathBuf, thread};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    plugin_file: Option<PathBuf>,

    #[arg(short, long, default_value = "./rumqttd.toml")]
    rumqttd_config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    pretty_env_logger::init();

    let config_file = config::File::from(args.rumqttd_config);

    // Config file is hardcoded to the current directory
    let config = config::Config::builder()
        .add_source(config_file)
        .build()
        .expect("couldn't open rumqttd config file")
        .try_deserialize()
        .expect("couldn't parse rumqttd config file");

    info!("-- create new Broker");
    let mut broker = Broker::new(config);

    info!("-- create broker link named 'pluginnode'");
    let (mut link_tx, link_rx) = broker.link("pluginnode").unwrap();

    info!("-- create broker link named 'monitornode'");
    let (mut monitor_link_tx, mut monitor_link_rx) = broker.link("monitornode").unwrap();

    info!("-- pluginnode subscribe to #");
    link_tx.subscribe("#").unwrap();
    info!("-- monitornode subscribe to #");
    monitor_link_tx.subscribe("#").unwrap();

    info!("-- start broker thread");
    thread::spawn(move || {
        broker.start().unwrap();
    });

    info!("-- start plugins");
    if let Some(plugin_filename) = args.plugin_file {
        let plugin = plugin::load_plugin(plugin_filename)?;
        start_plugin(plugin, link_tx, link_rx, "result".to_owned()).await?;
    }

    let mut count = 0;
    loop {
        let notification = match monitor_link_rx.recv().unwrap() {
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
