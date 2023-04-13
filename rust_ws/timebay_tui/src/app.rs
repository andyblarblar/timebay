//! Application state and logic

use crate::app::AppState::Connected;
use crate::mqtt::MqttClient;
use crate::splits::Splits;
use cursive::traits::Nameable;
use cursive::views::{Dialog, LinearLayout, TextView};
use derive_more::IsVariant;
use std::collections::BTreeSet;
use std::future::Future;
use std::sync::Arc;
use timebay_common::messages::{ConnectionMessage, DetectionMessage, DisconnectionMessage};

/// App connection state
#[derive(Debug, IsVariant, Clone)]
pub enum AppState {
    /// Connecting to MQTT
    Connecting,
    /// Connected to MQTT
    Connected { cli: Arc<MqttClient> },
}

/// Messages passed from external sources to be acted upon by the app
#[derive(Debug, Clone, IsVariant)]
pub enum AppMessage {
    /// Transition the system state
    StateChange(AppState),
    /// Sensor node connected
    ConnectNode(ConnectionMessage),
    /// Sensor node disconnected
    DisconnectNode(DisconnectionMessage),
    /// Vehicle detection
    Detection(DetectionMessage),
    /// Zero button was pressed
    SendZero,
    /// Zero op completed
    ZeroAck,
    /// Does nothing
    Nop,
}

/// Application state implementation
pub struct App {
    /// Current connection state of the app
    state: AppState,
    /// Connected sensor node ids
    connected_nodes: BTreeSet<u16>,
    /// Current lap we are timing
    lap: Splits,
    /// Last lap
    last_lap: Option<Splits>,
    /// Two laps ago. Not displayed, but used for diffs.
    last_last_lap: Option<Splits>,
}

impl App {
    /// Creates a new application
    pub fn new() -> Self {
        Self {
            state: AppState::Connecting,
            connected_nodes: BTreeSet::new(),
            lap: Splits::new(BTreeSet::new()),
            last_lap: None,
            last_last_lap: None,
        }
    }

    /// Generates the main body view based off current app state
    pub fn view(&self) -> impl cursive::view::View {
        if self.state.is_connecting() {
            LinearLayout::horizontal().child(Dialog::around(TextView::new("Connecting to gateway...")))
        } else {
            LinearLayout::horizontal()
                .child(
                    if let Some(ref last) = self.last_lap {
                        Dialog::around(last.view(&self.last_last_lap))
                    } else {
                        Dialog::around(TextView::new("Waiting for lap to complete..."))
                    }
                    .title("Last lap")
                    .with_name("last_lap"),
                )
                .child(
                    Dialog::around(self.lap.view(&self.last_lap))
                        .title("Current lap")
                        .with_name("current_lap"),
                )
                .child(
                    Dialog::around(
                        self.connected_nodes
                            .iter()
                            .fold(LinearLayout::horizontal(), |agg, n| {
                                agg.child(TextView::new(format!("| {} |", n)))
                            }),
                    )
                    .title("Connected sensors"),
                )
        }
    }

    /// Performs side effects based off an application message.
    ///
    /// These side effects are not long running to avoid blocking the gui.
    ///
    /// If an additional async side effect should occur, it can be returned. This side effect can in
    /// turn return another message.
    pub fn update(
        &mut self,
        message: AppMessage,
    ) -> Option<Box<dyn Future<Output = AppMessage> + Send>> {
        match message {
            AppMessage::StateChange(state) => {
                log::debug!("GUI state change to: {:?}", state);
                self.state = state;
            }
            AppMessage::ConnectNode(id) => {
                if self.connected_nodes.insert(id.node_id) {
                    log::info!("Sensor node: {} connected", id.node_id);
                } else {
                    log::debug!("Heartbeat connection from node {}", id.node_id);
                }

                // Add node to splits if we haven't started yet
                self.lap.connect_node(id.node_id);
            }
            AppMessage::DisconnectNode(id) => {
                log::info!("Sensor node: {} disconnected", id.node_id);
                if !self.connected_nodes.remove(&id.node_id) {
                    log::error!("Disconnected a non-connected sensor node!");
                }

                // Disconnect node from splits if we haven't begun yet
                self.lap.disconnect_node(id.node_id);
            }
            AppMessage::Detection(detc) => {
                if self.lap.handle_node_trigger(detc).is_completed() {
                    // Swap current lap to last lap when done
                    self.last_last_lap = self.last_lap.clone();
                    self.last_lap = Some(self.lap.clone());

                    // Start next lap immediately, since the first node overlaps both runs
                    self.lap = Splits::new(self.connected_nodes.clone());
                    self.lap.handle_node_trigger(detc);
                }
            }
            AppMessage::SendZero => {
                if let Connected { ref cli } = self.state {
                    log::trace!("sending zero mqtt");

                    // Send zero in a background thread and react on ack
                    let cli_cl = cli.clone();
                    return Some(Box::new(async move {
                        if cli_cl
                            .publish(timebay_common::messages::MqttMessage::Zero)
                            .await
                            .is_err()
                        {
                            AppMessage::StateChange(AppState::Connecting)
                        } else {
                            AppMessage::ZeroAck
                        }
                    }));
                }
            }
            AppMessage::ZeroAck => {
                log::trace!("Zero returned success");
            }
            AppMessage::Nop => {}
        };

        None
    }
}
