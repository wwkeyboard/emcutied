use std::{path::PathBuf, thread};

use anyhow::Result;
use log::info;
use rumqttd::{
    local::{LinkRx, LinkTx},
    Broker,
};

/// A wrapper for everything rumqttd, if this stays contained enough it could potentially be swapped out*.
///
///
/// * yes, I know how difficult and unlikely that is

pub struct Rumqttd {
    broker: Broker,
}
pub struct Link {
    pub link_tx: LinkTx,
    pub link_rx: LinkRx,
}

impl Rumqttd {
    pub fn new(config_file: PathBuf) -> Rumqttd {
        let config_file = config::File::from(config_file);

        // Config file is hardcoded to the current directory
        let rumqttd_config = config::Config::builder()
            .add_source(config_file)
            .build()
            .expect("couldn't open rumqttd config file")
            .try_deserialize()
            .expect("couldn't parse rumqttd config file");

        info!("-- create new Broker");
        Rumqttd {
            broker: Broker::new(rumqttd_config),
        }
    }

    pub fn link(&self, name: &str) -> Result<Link> {
        info!("-- create broker link named 'monitornode'");
        let (link_tx, link_rx) = self.broker.link(name).unwrap();

        Ok(Link { link_tx, link_rx })
    }

    pub fn start(mut self) {
        info!("-- start broker thread");
        thread::spawn(move || {
            self.broker.start().unwrap();
        });
    }
}
