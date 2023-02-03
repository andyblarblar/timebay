//! Messages sent over mqtt

use derive_more::Constructor;
use serde_derive::Deserialize;
use serde_derive::Serialize;

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
