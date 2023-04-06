mod application;
mod dist_sensor;
mod error;
mod handlers;
mod mqtt;
mod sensor_connection;

use crate::application::ApplicationContext;
use crate::handlers::{handle_mqtt_msg, handle_trigger};
use crate::mqtt::MqttClient;
use log::LevelFilter::Trace;
use simplelog::{ColorChoice, CombinedLogger, TerminalMode};
use tokio::join;

#[tokio::main]
async fn main() {
    CombinedLogger::init(vec![simplelog::TermLogger::new(
        Trace,
        simplelog::ConfigBuilder::default()
            .add_filter_ignore_str("paho_mqtt")
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    log::info!("Waiting for mqtt and sensor to connect...");

    let (sensor, mut client) = {
        // Get broker and node id from env vars
        let node_id = std::env::var("NODE_ID")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .unwrap();
        let server_host = std::env::var("BROKER_HOST").unwrap_or_else(|_| "localhost".to_string());

        let sensor_fut = sensor_connection::create_sensor();

        // Future that keeps polling until we connect to mqtt
        let client_fut = async {
            loop {
                if let Ok(conn) =
                    MqttClient::connect(node_id, &format!("mqtt://{}:1883", &server_host)).await
                {
                    log::info!("Successfully connected to broker");
                    break conn;
                } else {
                    log::error!("Timed out connecting to broker... (Do we have an IP on bat0?)");
                }
            }
        };
        join!(sensor_fut, client_fut)
    };
    log::info!("Sensor and mqtt both ready!");

    // Zero sensor
    let mut app = ApplicationContext::new(sensor, 10_000, 100);

    let zero = app.zero().await.expect("Failed initial zero!");
    log::info!("Set initial zero to: {}", zero);

    let mut disconnected = false;
    loop {
        // Attempt reconnect on disconnect
        if disconnected {
            log::error!("Disconnected from broker!");
            client.reconnect().await;
            log::info!("Reconnected to broker!");
            disconnected = false;
        }

        // Send connected messages as a sort of heartbeat, allowing for late connecting clients to discover us
        if client.pub_connected_msg().await.is_err() {
            disconnected = true;
            continue;
        }

        let rcv_fut = client.recv_mqtt_msg();
        let trg_fut = app.wait_for_trigger();

        // Accept new messages and wait for sensor concurrently (branches are mutually exclusive)
        tokio::select! {
            res = rcv_fut => {
                if res.is_err() {
                    disconnected = true;
                    continue
                }

                let msg = res.unwrap();
                if handle_mqtt_msg(msg, &mut client, &mut app).await.is_err() {
                    disconnected = true;
                }
            },
            res = trg_fut => {
                if res.is_err() {
                    log::error!("Sensor erred while reading!");
                    continue
                }
                if handle_trigger(&mut client, &mut app, res.unwrap()).await.is_err() {
                    disconnected = true;
                }
            }
        }
    }
}
