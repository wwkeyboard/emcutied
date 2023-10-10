use std::path::PathBuf;

use anyhow::Result;
use extism::{CurrentPlugin, Error, Function, UserData, Val, ValType};
use log::{debug, info, trace};
use rumqttd::{
    local::{LinkRx, LinkTx},
    Notification,
};

const PLUGIN_FUNCTION: &str = "handle";

pub struct Plugin {
    plugin: extism::Plugin<'static>,
}

pub fn load_plugin(path: PathBuf) -> Result<Plugin> {
    info!("Starting plugin: {path:?}");

    let wasm = std::fs::read(path)?;

    let f = Function::new("emit", [ValType::I64], [ValType::I64], None, host_emit);
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
                //                let _ = link_tx.publish(out_topic.to_owned(), res);
            }
            v => {
                trace!("plugin only handles forward notifications: {v:?}");
            }
        }
    }
}

fn host_emit(
    _plugin: &mut CurrentPlugin,
    inputs: &[Val],
    outputs: &mut [Val],
    _user_data: UserData,
) -> Result<(), Error> {
    println!("Hello from Rust's emit!");
    outputs[0] = inputs[0].clone();
    Ok(())
}
