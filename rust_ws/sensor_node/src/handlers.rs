use crate::application::ApplicationContext;
use crate::dist_sensor::{DistanceReading, DistanceSensor};
use crate::error::Error;
use crate::mqtt::MqttClient;
use std::time::{SystemTime, UNIX_EPOCH};
use timebay_common::messages::{DetectionMessage, MqttMessage};

/// Handles an incoming mqtt message
pub async fn handle_mqtt_msg<T: DistanceSensor>(
    msg: MqttMessage,
    _client: &mut MqttClient,
    ctx: &mut ApplicationContext<T>,
) -> Result<(), Error> {
    match msg {
        MqttMessage::Zero => {
            ctx.zero().await?;
            Ok(())
        }
        _ => Err(Error::WrongSub),
    }
}

/// Handles a trigger
pub async fn handle_trigger<T: DistanceSensor>(
    client: &mut MqttClient,
    _ctx: &mut ApplicationContext<T>,
    dist: DistanceReading,
) -> Result<(), Error> {
    // Publish a detection message
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Should never be earlier than epoch");
    let msg = MqttMessage::Detection(DetectionMessage::new(
        client.node_id(),
        dist.dist,
        now.as_secs(),
        now.subsec_nanos(),
    ));

    client.publish(msg).await?;

    Ok(())
}
