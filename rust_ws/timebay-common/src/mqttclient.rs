//! Shared MQTT client abstraction

use crate::error::MqttClientError as Error;
use crate::error::MqttClientError::ExplicitDisconnect;
use crate::messages::{MqttMessage, TOPICS};
use paho_mqtt::{AsyncClient, AsyncReceiver, ConnectOptions, CreateOptionsBuilder, Message};

/// Mqtt client abstraction.
///
/// This abstraction is threadsafe internally, so no extra locking is required around this struct
/// if being shared.
pub struct MqttClient {
    cli: AsyncClient,
    stream: AsyncReceiver<Option<Message>>,
}

impl MqttClient {
    /// Connects to MQTT broker with the configured options.
    pub async fn connect(
        server_id: &str,
        client_id: &str,
        subs: &[&str],
        opts: Option<ConnectOptions>,
    ) -> Result<Self, Error> {
        let mut client = CreateOptionsBuilder::new()
            .client_id(client_id)
            .server_uri(server_id)
            .create_client()?;

        client.connect(opts).await?;

        // Async sub stream
        let stream = client.get_stream(10);

        // Sub to topics
        let qoss: Vec<_> = subs.iter().map(|t| TOPICS[t]).collect();
        client.subscribe_many(subs, &qoss);

        //TODO make sure we have auto reconnect on

        Ok(Self {
            cli: client,
            stream,
        })
    }

    /// Spins until an mqtt message is received.
    pub async fn recv_mqtt_msg(&self) -> Result<MqttMessage, Error> {
        if let Some(msg) = self.stream.recv().await.unwrap() {
            log::trace!("Received MQTT message from topic: {}", msg.topic());
            Ok(msg.try_into()?)
        } else {
            Err(ExplicitDisconnect)
        }
    }

    /// Publishes a mqtt message.
    pub async fn publish(&self, msg: MqttMessage) -> Result<(), Error> {
        self.cli.publish(msg.try_into()?).await?;
        Ok(())
    }

    /// Attempts to reconnect to the broker, if not connected.
    pub async fn reconnect(&self) -> Result<(), Error> {
        if self.cli.is_connected() {
            return Ok(());
        }

        self.cli.reconnect().await?;

        Ok(())
    }
}
