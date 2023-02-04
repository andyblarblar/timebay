use crate::error::Error;
use crate::error::Error::MqttDisconnect;
use paho_mqtt::{AsyncClient, AsyncReceiver, ConnectOptionsBuilder, CreateOptionsBuilder, Message};
use timebay_common::messages::MqttMessage::{Connection, Disconnection};
use timebay_common::messages::{ConnectionMessage, DisconnectionMessage, MqttMessage, TOPICS};

pub struct MqttClient {
    cli: AsyncClient,
    node_id: u16,
    stream: AsyncReceiver<Option<Message>>,
}

impl MqttClient {
    /// Connects to MQTT broker, setting LWT and sending a connected message.
    pub async fn connect(node_id: u16, server_id: &str) -> Result<Self, Error> {
        let mut client = CreateOptionsBuilder::new()
            .client_id(format!("node{node_id}"))
            .server_uri(server_id)
            .create_client()?;

        // Set LWT
        let mut conn_opt = ConnectOptionsBuilder::default();
        let msg = Disconnection(DisconnectionMessage::new(node_id));
        conn_opt.will_message(msg.try_into()?);

        client.connect(conn_opt.finalize()).await?;

        // Async sub stream
        let stream = client.get_stream(10);

        // Sub to topics
        let topics = ["/zero"];
        let qoss: Vec<_> = topics.iter().map(|t| TOPICS[t]).collect();
        client.subscribe_many(&topics, &qoss);

        // Pub connected message
        let msg = Connection(ConnectionMessage::new(node_id));
        client.publish(msg.try_into()?).await?;

        Ok(Self {
            cli: client,
            node_id,
            stream,
        })
    }
    
    /// Spins until an mqtt message is received. 
    pub async fn recv_mqtt_msg(&mut self) -> Result<MqttMessage, Error> {
        if let Some(msg) = self.stream.recv().await.unwrap() {
            Ok(msg.try_into()?)
        } else {
            Err(MqttDisconnect)
        }
    }
}
