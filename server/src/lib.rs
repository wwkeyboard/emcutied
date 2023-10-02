use anyhow::Result;
use extism::Plugin;
use log::{trace, debug, error, info};
use rumqttd::{
    local::{LinkRx, LinkTx},
    Notification,
};

const PLUGIN_FUNCTION: &str = "handle";

pub async fn start_plugin<'a>(
    mut plugin: Plugin<'_>,
    mut link_tx: LinkTx,
    mut link_rx: LinkRx,
    out_topic: String,
) -> Result<()> {
    info!("Starting plugin ---------------------");
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
                let res: Vec<u8> = plugin.call(PLUGIN_FUNCTION, payload)?.into();

                plugin.cancel_handle().cancel();

                trace!("-- result {:?}", &res);
//                let _ = link_tx.publish(out_topic.to_owned(), res);
            }
            v => {
                error!("unknown plugin notification: {v:?}");
            }
        }
    }
}
