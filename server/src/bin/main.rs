use anyhow::Result;
use clap::Parser;
use rumqttd::{Broker, Config};
use server::start_plugins;

use std::{path::PathBuf, thread};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    plugin_file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let builder = tracing_subscriber::fmt()
        .pretty()
        .with_line_number(false)
        .with_file(false)
        .with_thread_ids(false)
        .with_thread_names(false);

    builder
        .try_init()
        .expect("initialized subscriber succesfully");

    // Config file is hardcoded to the current directory
    let config = config::Config::builder()
        .add_source(config::File::with_name("rumqttd.toml"))
        .build()
        .unwrap();

    let config: Config = config.try_deserialize().unwrap();

    //    dbg!(&config);

    let mut broker = Broker::new(config);
    let (link_tx, link_rx) = broker.link("singlenode").unwrap();

    thread::spawn(move || {
        broker.start().unwrap();
    });

    if let Some(plugin_filename) = args.plugin_file {
        start_plugins(plugin_filename, link_tx, link_rx)?;
    }
    Ok(())
}
