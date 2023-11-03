use std::collections::HashMap;

use crate::plugin::Plugin;

/// Router
struct Router {
    plugins: Plugins,
}

type Plugins = HashMap<String, Vec<Box<Plugin>>>;

impl Router {
    pub fn new() -> Router {
        Router {
            plugins: HashMap::new(),
        }
    }

    pub fn add(&mut self, plugin: Box<Plugin>) {
        let key = plugin.in_topic.clone();
        let ps = self.plugins.entry(key).or_insert(Vec::new());
        ps.push(plugin);
    }
    // TODO: Add function that takes a plugin and adds it to the plugins hash
    // TODO: route function that takes a message and figures out which plugin to send it to
    // TODO: a run function that takes the link_rx, pulls messages from it, asks the route
    // where it should go, then spawns a new tokio task to handle that plugin
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
        let plugin = Plugin::new(path, "/intopic", "/outtopic").unwrap();

        router.add(Box::new(plugin));
        assert_eq!(1, router.plugins.len());
    }
    fn add_appends_to_existing_vec() {}
}
