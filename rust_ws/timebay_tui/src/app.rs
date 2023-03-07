use crate::mqtt::MqttClient;
use crate::splits::Splits;
use cursive::traits::Nameable;
use cursive::views::{Dialog, LinearLayout, TextView};
use derive_more::IsVariant;
use std::collections::BTreeSet;
use std::sync::Arc;
use timebay_common::messages::{ConnectionMessage, DetectionMessage, DisconnectionMessage};

#[derive(Debug, IsVariant, Clone)]
pub enum AppState {
    /// Connecting to MQTT
    Connecting,
    /// Connected to MQTT
    Connected { cli: Arc<MqttClient> },
}

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
            LinearLayout::horizontal().child(Dialog::around(TextView::new("Connecting...")))
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

    /// Performs side effects based off an application message
    pub fn update(&mut self, message: AppMessage) {
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
            }
            AppMessage::Detection(detc) => {
                if self.lap.handle_node_trigger(detc).is_completed() {
                    // Swap current lap to last lap when done
                    self.last_last_lap = self.last_lap.clone();
                    self.last_lap = Some(self.lap.clone());
                    self.lap = Splits::new(self.connected_nodes.clone());
                }
            }
            AppMessage::SendZero => { /* This should be handled in the eventloop */ }
            AppMessage::ZeroAck => {
                log::info!("Zero returned success");
                
            }
        };
    }
}
