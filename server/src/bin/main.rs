use anyhow::Result;
use clap::Parser;
use extism::Plugin;
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

    let builder = tracing_subscriber::fmt()
        .pretty()
        .with_line_number(false)
        .with_file(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    builder
        .try_init()
        .expect("initialized subscriber successfully");

    // Config file is hardcoded to the current directory
    let config = config::Config::builder()
        .add_source(config::File::with_name("rumqttd.toml"))
        .build()
        .unwrap();

    let config: Config = config.try_deserialize().unwrap();

    let mut broker = Broker::new(config);
    let (mut link_tx, link_rx) = broker.link("pluginnode").unwrap();

    let (mut monitor_link_tx, mut monitor_link_rx) = broker.link("monitornode").unwrap();

    link_tx.subscribe("#").unwrap();
    monitor_link_tx.subscribe("#").unwrap();

    thread::spawn(move || {
        broker.start().unwrap();
    });

    if let Some(plugin_filename) = args.plugin_file {
        println!("Starting plugin: {plugin_filename:?}");
        let wasm = std::fs::read(plugin_filename.clone()).unwrap();
        let plugin = Plugin::new(wasm, [], false).unwrap();
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
                println!("{v:?}");
            }
        }
    }
}
