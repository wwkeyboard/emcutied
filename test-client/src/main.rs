use rumqttc::{Client, MqttOptions, QoS};
use std::{thread, time::Duration};

/*
 * Mostly cribbed from https://nikolas.blog/how-to-use-rust-and-mqtt-in-your-next-project/
 */

fn main() {
    // the client name must be unique on the broker
    let mut mqttoptions = MqttOptions::new("test-client", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);
    client.subscribe("demo/mqtt", QoS::AtMostOnce).unwrap();

    thread::spawn(move || {
        for i in 0..10 {
            client
                .publish("hello/world", QoS::AtLeastOnce, false, vec![i; i as usize])
                .unwrap();
            thread::sleep(Duration::from_millis(100));
        }
    });

    for (_i, message) in connection.iter().enumerate() {
        println!("Message= {:?}", message);
    }
}
