use crate::error::Error;
use paho_mqtt::{AsyncClient, ConnectOptionsBuilder, CreateOptionsBuilder, Message};
use timebay_common::messages::{ConnectionMessage, DisconnectionMessage};

pub struct MqttClient {
    cli: AsyncClient,
    node_id: u16,
}

impl MqttClient {
    /// Connects to MQTT broker, setting LWT and sending a connected message.
    pub async fn connect(node_id: u16, server_id: &str) -> Result<Self, Error> {
        let client = CreateOptionsBuilder::new()
            .client_id(format!("node{node_id}"))
            .server_uri(server_id)
            .create_client()?;

        // Set LWT
        let mut conn_opt = ConnectOptionsBuilder::default();
        let msg = DisconnectionMessage::new(node_id);
        let msg = postcard::to_allocvec(&msg)?;
        let lwt = Message::new("/disconnect", msg, 2);
        conn_opt.will_message(lwt);

        client.connect(conn_opt.finalize()).await?;

        // Pub connected message
        let msg = ConnectionMessage::new(node_id);
        let msg = postcard::to_allocvec(&msg)?;
        let conn_msg = Message::new("/connect", msg, 2);
        client.publish(conn_msg).await?;

        Ok(Self {
            cli: client,
            node_id,
        })
    }
}
