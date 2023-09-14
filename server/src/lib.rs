use std::path::PathBuf;

use anyhow::Result;
use extism::{Context, Function, Plugin};
use rumqttd::{
    local::{LinkRx, LinkTx},
    Notification,
};

pub fn start_plugins(plugin_file: PathBuf, mut link_tx: LinkTx, mut link_rx: LinkRx) -> Result<()> {
    // load the plugin
    let wasm = std::fs::read(plugin_file)?;
    let context = Context::new();
    let functions = std::iter::empty::<Function>();
    let mut _plugin = Plugin::new(&context, wasm, functions, false).unwrap();

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
