use rumqttc::{AsyncClient, MqttOptions, QoS, Event};
use std::error::Error;
use std::time::Duration;

pub async fn publish_message(
    broker_host: &str,
    broker_port: u16,
    topic: &str,
    message: &str,
    qos: QoS,
) -> Result<(), Box<dyn Error>> {
    let mut mqtt_options = MqttOptions::new("mqtt-sender", broker_host, broker_port);
    mqtt_options.set_keep_alive(Duration::from_secs(10));
    
    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    println!("(publisher thread) Connecting to broker {}:{}", broker_host, broker_port);
    println!("(publisher thread) Publishing message '{}' to topic '{}'", message, topic);
    
    client.publish(topic, qos, false, message.as_bytes()).await?;

    loop {
        match eventloop.poll().await {
            Ok(Event::Outgoing(_)) => {
                println!("(publisher thread) message successfully published");
                break;
            }
            Ok(other) => println!("(publisher thread) some other event received: {:?}", other),
            Err(e) => println!("(publisher thread) Error: {:?}", e),
        }
    }
    Ok(())
}
