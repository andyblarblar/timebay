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

#[derive(Debug, Clone)]
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

pub struct App {
    /// Current connection state of the app
    state: AppState,
    /// Connected sensor node ids
    connected_nodes: BTreeSet<u16>,
    /// Current lap we are timing
    lap: Splits,
    /// Last lap
    last_lap: Option<Splits>,
    /// Last last lap
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
    } //TODO I think we can just have app create the full view as in iced, and have the app live in the other thread. This thread polls both over zeroed commands from the GUI, MQTT messages, and then sets the full view via cb_sink
      //This way we can directly port update and view. view is just always called from update, although we need to figure out how to emulate message passing.

    /// Generates the main body view
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
}
