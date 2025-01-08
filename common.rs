use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::error::Error;
use std::time::Duration;

pub const BROKER_HOST: &str = "localhost";
pub const BROKER_PORT: u16 = 1883;
pub const TOPIC: &str = "my/topic";
pub const QOS: QoS = QoS::AtLeastOnce;

pub async fn create_mqtt_client() -> Result<(AsyncClient, tokio::task::JoinHandle<()>), Box<dyn Error>> {
    let mut mqtt_options = MqttOptions::new("mqtt-client", BROKER_HOST, BROKER_PORT);
    mqtt_options.set_keep_alive(Duration::from_secs(10));

    let (client, mut eventloop) = AsyncClient::new(mqtt_options, 10);

    let eventloop_task = tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(event) => log::info!("MQTT event: {:?}", event),
                Err(e) => log::error!("MQTT error: {:?}", e),
            }
        }
    });

    Ok((client, eventloop_task))
}
