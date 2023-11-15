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