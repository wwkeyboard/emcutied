use clap::Parser;
use rumqttc::{Client, MqttOptions, QoS};
use std::{thread, time::Duration};

/*
 * Mostly cribbed from https://nikolas.blog/how-to-use-rust-and-mqtt-in-your-next-project/
 */

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// topic to publish payload on
    #[arg(short, long)]
    publish_topic: String,

    /// topic to listen to, defaults to #
    #[arg(short, long)]
    listen_topic: Option<String>,

    /// payload to publish
    publish_payload: String,
}

fn main() {
    let cli = Cli::parse();

    // the client name must be unique on the broker
    let mut mqttoptions = MqttOptions::new("test-client", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    if let Some(topic) = cli.listen_topic {
        client.subscribe(topic, QoS::AtMostOnce).unwrap();
    }

    client
        .publish(
            cli.publish_topic,
            QoS::AtLeastOnce,
            false,
            cli.publish_payload.as_bytes(),
        )
        .unwrap();
    thread::sleep(Duration::from_millis(100));

    for (_i, message) in connection.iter().enumerate() {
        match message {
            Ok(msg) => {
                match msg {
                    rumqttc::Event::Incoming(inmsg) => {
                        println!("got = {:?}", inmsg);
                    }
                    rumqttc::Event::Outgoing(omsg) => {
                        println!("got = {:?}", omsg); // noop
                    }
                }
            }
            Err(e) => {
                println!("ERROR => {:?}", e);
                return;
            }
        }
    }
}
