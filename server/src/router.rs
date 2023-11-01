use std::collections::HashMap;

use crate::{plugin::Plugin, rumqttd::Link};

/// Router
struct Router {
    links: Link,
    plugins: Plugins,
}

type Plugins = HashMap<Box<Plugin>>;

impl Router {
    pub fn new(links: Links, plugins: Plugins) -> Router {
        Router { links, plugins }
    }
}
