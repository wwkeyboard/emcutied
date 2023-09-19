use std::path::PathBuf;

use anyhow::{ Result};
use extism::{Plugin};
use rumqttd::{
    local::{LinkRx, LinkTx},
    Notification,
};

const PLUGIN_FUNCTION: &str = "handle";


pub async fn start_plugin<'a>(
    mut plugin: Plugin,
    mut link_tx: LinkTx,
    mut link_rx: LinkRx,
    out_topic: String,
) -> Result<()> {
    println!("Starting plugin ---------------------");
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
                println!(
                    ">>> Topic = {:?}, Count = {}, Payload = {} bytes",
                    forward.publish.topic,
                    count,
                    forward.publish.payload.len()
                );

                let payload:Vec<u8> = forward.publish.payload.to_vec();
                let res:Vec<u8> = plugin.call(PLUGIN_FUNCTION, payload)?;

                plugin.cancel_handle().cancel()?;

                dbg!(&res);
                let _ = link_tx.publish(out_topic.to_owned(), res);
            }
            v => {
                println!("unknown plugin notification: {v:?}");
            }
        }
    }
}
