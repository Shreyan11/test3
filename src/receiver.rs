use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::error::Error;
use std::time::Duration;

pub async fn subscribe_to_topic(
    broker_host: &str,
    broker_port: u16,
    topic: &str,
    qos: QoS,
) -> Result<(), Box<dyn Error>> {
    let mut mqtt_options = MqttOptions::new("mqtt-receiver", broker_host, broker_port);
    mqtt_options.set_keep_alive(Duration::from_secs(10));
    
    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    println!("(subscriber thread) connecting to broker {}:{}", broker_host, broker_port);
    println!("(subscriber thread) subscribing to topic '{}'", topic);
    
    client.subscribe(topic, qos).await?;

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                let payload = String::from_utf8_lossy(&publish.payload);
                println!("(subscriber thread) received message on topic '{}': '{}'", publish.topic, payload);
            }
            Ok(other) => println!("(subscriber thread) Other event received: {:?}", other),
            Err(e) => println!("(subscriber thread) Error: {:?}", e),
        }
    }
}
