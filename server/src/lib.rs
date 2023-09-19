use std::path::PathBuf;

use anyhow::Result;
use extism::{Context, Plugin};
use rumqttd::{
    local::{LinkRx, LinkTx},
    Notification,
};

static mut CONTEXT: &Context = &Context::new();

pub fn start_plugins(plugin_file: PathBuf, mut link_tx: LinkTx, mut link_rx: LinkRx) -> Result<()> {
    // load the plugin
    let wasm = std::fs::read(plugin_file)?;

    link_tx.subscribe("#").unwrap();

    let mut count = 0;
    loop {
        let notification = match link_rx.recv().unwrap() {
            Some(v) => v,
            None => continue,
        };

        match notification {
            Notification::Forward(forward) => {
                count += 1;
                println!(
                    ">>> Topic = {:?}, Count = {}, Payload = {} bytes",
                    forward.publish.topic,
                    count,
                    forward.publish.payload.len()
                );

                //let functions = std::iter::empty::<Function>();
                let mut plugin = Plugin::new(CONTEXT, &wasm, [], false).unwrap();
                let res = plugin.call("handle", forward.publish.payload)?;
                link_tx.publish("response", res)?;
            }
            v => {
                println!("{v:?}");
            }
        }
    }
}
