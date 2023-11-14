use std::collections::HashMap;
use std::sync::Mutex;

use crate::plugin::Plugin;
use log::{error, trace, warn};
use rumqttd::local::LinkRx;

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

    pub fn run(self, mut link_rx: LinkRx) -> ! {
        loop {
            let notification = match link_rx.recv().unwrap() {
                Some(v) => v,
                None => todo!(),
            };

            match notification {
                rumqttd::Notification::Forward(message) => {
                    trace!("received message on {:?}", message.publish.topic);
                    let Ok(topic) = std::str::from_utf8(&message.publish.topic) else {
                        error!("couldn't parse topic from message");
                        continue;
                    };

                    let Ok(payload) = std::str::from_utf8(&message.publish.payload) else {
                        error!("we only handle utf8 JSON payloads");
                        continue;
                    };

                    self.route(topic.to_owned(), payload.to_owned())
                }
                v => {
                    warn!("unhandled message {v:?}");
                }
            }
        }
    }

    fn route(&self, topic: String, message: String) {
        let Some(plugins) = self.plugins.get(&topic) else {
            return;
        };
        for plugin in plugins {
            let Ok(mut plugin) = plugin.lock() else {
                error!("couldn't lock a plugin on topic {topic}",);
                continue;
            };
            let Ok(res) = plugin.run(&message) else {
                warn!("error running plugin");
                continue;
            };
            println!(">> {res:?}");
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

    #[test]
    fn add_appends_to_existing_vec() {}
}
