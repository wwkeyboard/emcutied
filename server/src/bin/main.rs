use anyhow::Result;
use clap::Parser;
use extism::{Context, Plugin};
use log::{info, warn};
use pretty_env_logger;
use rumqttd::{Broker, Config, Notification};
use server::start_plugin;

use std::{path::PathBuf, thread};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    plugin_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    pretty_env_logger::init();

    // let builder = tracing_subscriber::fmt()
    //     .pretty()
    //     .with_line_number(false)
    //     .with_file(false)
    //     .with_thread_ids(false)
    //     .with_thread_names(false);

    // builder
    //     .try_init()
    //     .expect("initialized subscriber successfully");

    // Config file is hardcoded to the current directory
    let config = config::Config::builder()
        .add_source(config::File::with_name("rumqttd.toml"))
        .build()
        .unwrap();

    let config: Config = config.try_deserialize().unwrap();

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
        info!("Starting plugin: {plugin_filename:?}");
        let wasm = std::fs::read(plugin_filename.clone()).unwrap();
        let ctx = Context::new();
        let plugin = Plugin::new(&ctx, wasm, [], false).unwrap();
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
