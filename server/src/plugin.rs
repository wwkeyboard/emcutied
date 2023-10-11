use std::path::PathBuf;

use anyhow::Result;
use extism::{CurrentPlugin, Error, Function, InternalExt, UserData, Val, ValType};
use log::{debug, info, trace};
use rumqttd::{
    local::{LinkRx, LinkTx},
    Notification,
};

const PLUGIN_FUNCTION: &str = "handle";
const OUT_TOPIC: &str = "result";

pub struct Plugin {
    plugin: extism::Plugin<'static>,
}

pub fn load_plugin(path: PathBuf, link_tx: LinkTx) -> Result<Plugin> {
    info!("Starting plugin: {path:?}");

    let wasm = std::fs::read(path)?;

    let user_data = UserData::new(Box::new(link_tx));
    let f = Function::new("emit", [ValType::I64], [], Some(user_data), host_emit);
    let functions = [f];

    let plugin = extism::Plugin::create(wasm, functions, false)?;

    Ok(Plugin { plugin })
}

pub async fn start_plugin<'a>(
    mut plugin: Plugin,
    mut link_tx: LinkTx,
    mut link_rx: LinkRx,
    _out_topic: String,
) -> Result<()> {
    info!("Starting plugin ---------------------");

    // This looks a little wonkey, but you have to tx the subscription message on the tx link,
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

fn host_emit(
    plugin: &mut CurrentPlugin,
    inputs: &[Val],
    _outputs: &mut [Val],
    mut user_data: UserData,
) -> Result<(), Error> {
    let payload = plugin.memory_read_str(inputs[0].unwrap_i64() as u64)?;

    let data = payload.clone();

    if let Some(link_tx) = user_data.any_mut() {
        let tx = link_tx.downcast_mut::<LinkTx>().unwrap();

        let _ = tx.try_publish(OUT_TOPIC.to_owned(), payload);
    }

    println!("Hello from Rust's emit! sending payload {payload:?}");
    Ok(())
}
