use crate::application::ApplicationContext;
use crate::dist_sensor::DistanceSensor;
use crate::error::Error;
use crate::mqtt::MqttClient;
use timebay_common::messages::MqttMessage;

/// Handles an incoming mqtt message
pub async fn handle_mqtt_msg<T: DistanceSensor>(
    msg: MqttMessage,
    client: &mut MqttClient,
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
    ctx: &mut ApplicationContext<T>,
) -> Result<(), Error> {
    todo!("Pub to triggered topic")
    // Note that we will need to sleep for a few seconds after a trigger to debounce
}
