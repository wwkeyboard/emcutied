use std::path::PathBuf;

use anyhow::{Context, Result};
use log::{debug, info, trace};
use rumqttd::{
    local::{LinkRx, LinkTx},
    Notification,
};

const PLUGIN_FUNCTION: &str = "handle";

pub struct Plugin {
    plugin: extism::Plugin<'static>,
    pub in_topic: String,
    pub out_topic: String,
    pub file: PathBuf,
}

impl Plugin {
    pub fn new(file: PathBuf, in_topic: &str, out_topic: &str) -> Result<Plugin> {
        // load from file
        let plugin = Plugin::load_plugin(&file)?;

        // get links from broker
        let p = Plugin {
            plugin,
            in_topic: in_topic.to_owned(),
            out_topic: out_topic.to_owned(),
            file,
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

    // TODO: pull this out into a router that listens to everything and
    // dispatches to the correct plugin.
    // pub async fn run(mut self) -> Result<()> {
    //     info!(
    //         "Starting plugin: {}",
    //         self.file.to_str().unwrap_or("unknown")
    //     );
    //     self.links.link_tx.subscribe(self.in_topic);
    //
    //     loop {
    //         let notification = match self.links.link_rx.recv()? {
    //             Some(v) => v,
    //             None => return Ok(()), // all senders have been dropped inside rumqttd
    //         };
    //
    //         match notification {
    //             Notification::Forward(forward) => {
    //                 let payload: Vec<u8> = forward.publish.payload.to_vec();
    //                 debug!(
    //                     ">>> Topic = {:?}, Payload = {}",
    //                     forward.publish.topic,
    //                     String::from_utf8(payload.clone())
    //                         .unwrap_or("<<not printable>>".to_owned())
    //                 );
    //
    //                 let res: Vec<u8> = self.plugin.call(PLUGIN_FUNCTION, payload)?.into();
    //
    //                 self.plugin.cancel_handle().cancel();
    //
    //                 trace!("-- result {:?}", &res);
    //             }
    //             v => {
    //                 trace!("plugin only handles forward notifications: {v:?}");
    //             }
    //         }
    //     }
    // }
}

pub async fn start_plugin<'a>(
    mut plugin: Plugin,
    mut link_tx: LinkTx,
    mut link_rx: LinkRx,
    _out_topic: String,
) -> Result<()> {
    info!("Starting plugin ---------------------");

    // This looks a little wonkey, but you have to send the subscription message on the tx link,
    // not the rx link.
    link_tx.subscribe("doubler").unwrap();

    let mut count = 0;

    loop {
        let notification = match link_rx.recv().unwrap() {
            Some(v) => v,
            None => return Ok(()),
        };

        match notification {
            Notification::Forward(forward) => {
                count += 1;
                debug!(
                    ">>> Topic = {:?}, Count = {}, Payload = {} bytes",
                    forward.publish.topic,
                    count,
                    forward.publish.payload.len()
                );

                let payload: Vec<u8> = forward.publish.payload.to_vec();
                let res: Vec<u8> = plugin.plugin.call(PLUGIN_FUNCTION, payload)?.into();

                plugin.plugin.cancel_handle().cancel();

                trace!("-- result {:?}", &res);
            }
            v => {
                trace!("plugin only handles forward notifications: {v:?}");
            }
        }
    }
}
