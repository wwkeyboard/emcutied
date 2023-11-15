# emcutied
MQTT broker that allows WASM plugins to listen and respond to topics.

# Buliding the plugin

There is an example plugin that listens to a topic, parses json messages, and doubles a number. It's creatively called `double-plugin` and it can be built by changing to the double-plugin directory and running:

    cargo build --release --target wasm32-unknown-unknown

The resulting wasm will be in the project's root `target/wasm32-unknown-unknown/release/double_plugin.wasm`

# Running the broker

rumqttd and wasmtime can be very noisy, often to the point of needing to limit logging to only the module you're concerned with.

    RUST_LOG="server::router=trace" cargo run -- -c ./emcutid_config.toml

# Architecture 

PluginConfig is produced from the emcutid config file, it contains the path to the rumqttd config(which eventually should be inlined so there is only one config file to worry about), it also contains the list of plugins to run. The rumqttd config us used to set up the rumqttd broker, the plugins are all instantiated and added to the plugin router, and a link from the broker is given to the router. Then the broker is started, which runs in it's own tokio task, and then the plugin router is started. At the moment the plugin router is a big loop that takes control of everything(this is also on the short list of things to fix).

```
                  ┌──────────────┐
                  │              │
        ┌─────────┤ PluginConfig ├────────┐
        │         │              │        │
        │         └──────────────┘        │
        │                                 │
        │                                 │
        │                                 │
┌───────▼────────┐                 ┌──────▼────────┐
│                ├────────────────►│               │
│ rumqttd broker │  LinkTx/LinkRx  │ Plugin Router │
│                │◄────────────────┤               │
└────────────────┘                 └┬──────────────┘
                                    │             ▲
                                    │             │
                                    │  ┌────────┐ │
                                    │  │ Plugin │ │
                                    │  ├────────┤ │
                                    └─►│ Plugin ├─┘
                                       ├────────┤
                                       │ Plugin │
                                       ├────────┤
                                       │ Plugin │
                                       └────────┘
```