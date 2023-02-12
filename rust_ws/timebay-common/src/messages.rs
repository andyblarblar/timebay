//! Messages sent over mqtt

use crate::error::ConversionError;
use crate::error::ConversionError::NonConvertable;
use crate::messages::MqttMessage::{Unknown, Zero};
use derive_more::{Constructor, From, IsVariant, TryInto, Unwrap};
use paho_mqtt::Message;
use phf::phf_map;
use serde_derive::Deserialize;
use serde_derive::Serialize;

/// Maps topics to QoS
pub static TOPICS: phf::Map<&'static str, i32> = phf_map! {
    "/connect" => 2,
    "/disconnect" => 2,
    "/zero" => 1,
    "/sensors/detection" => 2,
};

/// All possible timebay messages.
///
/// This type is designed to be used with `try_from` and `try_into` to covert raw Mqtt messages into their
/// payloads content as a struct.
///
/// Ex:
///
/// ```rust
/// use paho_mqtt::Message;
/// use timebay_common::messages::{ConnectionMessage, MqttMessage};
/// use timebay_common::messages::MqttMessage::Connection;
///
/// let msg: Message = Connection(ConnectionMessage::new(1)).try_into().unwrap();
///
///         assert_eq!(
///             MqttMessage::try_from(msg)
///                 .unwrap()
///                 .unwrap_connection()
///                 .node_id,
///             1
///         );
///
///
/// ```
#[derive(Unwrap, Debug, From, IsVariant, TryInto, Clone, Eq, PartialEq)]
pub enum MqttMessage {
    /// A node was connected
    Connection(ConnectionMessage),
    /// A node was disconnected
    Disconnection(DisconnectionMessage),
    /// A node detected the vehicle
    Detection(DetectionMessage),
    /// Zeros all sensors
    Zero,
    /// A message on an unknown topic
    #[try_into(ignore)]
    Unknown(String),
}

// Received message into enum
impl TryFrom<Message> for MqttMessage {
    type Error = ConversionError;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value.topic() {
            "/connect" => Ok(postcard::from_bytes::<ConnectionMessage>(value.payload())?.into()),
            "/disconnect" => {
                Ok(postcard::from_bytes::<DisconnectionMessage>(value.payload())?.into())
            }
            "/zero" => Ok(Zero),
            "/sensors/detection" => {
                Ok(postcard::from_bytes::<DetectionMessage>(value.payload())?.into())
            }

            _ => Ok(Unknown(value.topic().into())),
        }
    }
}

// enum into sendable message
impl TryFrom<MqttMessage> for Message {
    type Error = ConversionError;

    fn try_from(value: MqttMessage) -> Result<Self, Self::Error> {
        match value {
            MqttMessage::Connection(conn) => Ok(Message::new(
                "/connect",
                postcard::to_allocvec(&conn)?,
                TOPICS["/connect"],
            )),
            MqttMessage::Disconnection(conn) => Ok(Message::new(
                "/disconnect",
                postcard::to_allocvec(&conn)?,
                TOPICS["/disconnect"],
            )),
            MqttMessage::Detection(det) => Ok(Message::new(
                "/sensors/detection",
                postcard::to_allocvec(&det)?,
                TOPICS["/sensors/detection"],
            )),
            MqttMessage::Zero => Ok(Message::new("/zero", [], TOPICS["/zero"])),
            Unknown(_) => Err(NonConvertable),
        }
    }
}

/// Message published when node connects to broker
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq, Copy, Clone, Constructor)]
pub struct ConnectionMessage {
    pub node_id: u16,
}

/// Message published when node disconnects from broker
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq, Copy, Clone, Constructor)]
pub struct DisconnectionMessage {
    pub node_id: u16,
}

/// Message published on vehicle detection.
#[derive(Serialize, Deserialize, Debug, Default, Eq, PartialEq, Copy, Clone, Constructor)]
pub struct DetectionMessage {
    pub node_id: u16,
    /// Detection distance in mm.
    pub dist: u32,
    /// Detection time in unix seconds.
    pub stamp_s: u64,
    /// Nanoseconds fraction of the unix stamp.
    pub stamp_ns: u32,
}

#[cfg(test)]
mod test {
    use crate::messages::MqttMessage::{Connection, Detection, Disconnection, Zero};
    use crate::messages::{ConnectionMessage, DetectionMessage, DisconnectionMessage, MqttMessage};
    use paho_mqtt::Message;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn mqtt_message_parses() {
        let msg: Message = Connection(ConnectionMessage::new(1)).try_into().unwrap();

        assert_eq!(
            MqttMessage::try_from(msg)
                .unwrap()
                .unwrap_connection()
                .node_id,
            1
        );

        let msg: Message = Disconnection(DisconnectionMessage::new(1))
            .try_into()
            .unwrap();

        assert_eq!(
            MqttMessage::try_from(msg)
                .unwrap()
                .unwrap_disconnection()
                .node_id,
            1
        );

        let msg: Message = Zero.try_into().unwrap();
        assert!(MqttMessage::try_from(msg).unwrap().is_zero());

        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let detect_msg = Detection(DetectionMessage::new(
            2,
            523,
            time.as_secs(),
            time.subsec_nanos(),
        ));
        let msg: Message = detect_msg.clone().try_into().unwrap();

        assert_eq!(
            MqttMessage::try_from(msg).unwrap().unwrap_detection(),
            detect_msg.unwrap_detection()
        );
    }
}
