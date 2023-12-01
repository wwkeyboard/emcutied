use std::collections::HashMap;
use std::sync::Mutex;

use crate::config;
use crate::plugin::Plugin;
use crate::rumqttd::Link;
use anyhow::Result;

use log::{error, trace, warn};

/// Router
pub struct Router {
    plugins: Plugins,
}

type Plugins = HashMap<String, Vec<Mutex<Box<Plugin>>>>;

impl Router {
    pub fn new() -> Router {
        Router {
            plugins: HashMap::new(),
        }
    }

    pub fn add_new_plugins(&mut self, plugins: Vec<config::Plugin>) -> Result<()> {
        for plugin_config in plugins {
            let plugin = Plugin::new(
                plugin_config.file,
                plugin_config.name.as_str(),
                plugin_config.in_topic.as_str(),
                plugin_config.out_topic.as_str(),
            )?;

            self.add(Box::new(plugin));
        }
        Ok(())
    }

    pub fn add(&mut self, plugin: Box<Plugin>) {
        let key = plugin.in_topic.clone();
        let ps = self.plugins.entry(key).or_default();

        trace!(
            "on topics {} -> {} loading plugin {}",
            plugin.in_topic,
            plugin.out_topic,
            plugin.name
        );

        ps.push(Mutex::new(plugin));
    }

    /// Main run loop for the router. Wait for a notification from the given
    /// link, then extract a message from it, look for any plugins that would
    /// handle that message, and then call them.
    ///
    /// # Arguments
    ///
    /// * `link` - The link to listen for notifications and send responses on.
    pub fn run(self, mut link: Link) -> ! {
        loop {
            let notification = match link.link_rx.recv().unwrap() {
                Some(v) => v,
                None => todo!(),
            };

            match notification {
                rumqttd::Notification::Forward(message) => {
                    trace!("received message on {:?}", message.publish.topic);

                    let Ok(topic) = std::str::from_utf8(&message.publish.topic) else {
                        error!("couldn't parse topic from message ");
                        continue;
                    };

                    let Ok(payload) = std::str::from_utf8(&message.publish.payload) else {
                        error!("couldn't parse payload from message");
                        continue;
                    };

                    if let Some(plugins) = self.plugins.get(topic) {
                        call_plugins(plugins, payload, &mut link);
                    };
                }
                v => {
                    warn!("unhandled message {v:?}");
                }
            }
        }
    }
}

/// Calls each of the given plugins with the given payload and sends any response on the link.
///
/// # Arguments
///
/// * `plugins` - The plugins to call.
/// * `payload` - The payload to pass to the plugins.
/// * `link` - The Link to respond on.
///
/// # Example
///
/// ```
/// let plugins = vec![Mutex::new(Box::new(plugin))];
/// let payload = "example payload";
/// let mut link = Link::new();
///
/// call_plugins(&plugins, payload, &mut link);
/// ```
fn call_plugins(plugins: &Vec<Mutex<Box<Plugin>>>, payload: &str, link: &mut Link) {
    for plugin in plugins {
        if let Ok(mut plugin) = plugin.lock() {
            trace!("calling plugin {}", plugin.name);
            if let Ok(result) = plugin.run(payload) {
                trace!("sending result {:?}", result);
                if let Err(e) = link.link_tx.publish(plugin.out_topic.to_owned(), result) {
                    error!("error sending result {:?}", e);
                };
            }
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn add_creates_new_vec_then_appends() {
        let mut router = Router::new();

        // TODO: yes this is terrible
        let path = PathBuf::from("../double_plugin.wasm");
        let plugin = Plugin::new(path, "test-plugin", "/intopic", "/outtopic").unwrap();

        router.add(Box::new(plugin));
        assert_eq!(1, router.plugins.len());
    }
}
