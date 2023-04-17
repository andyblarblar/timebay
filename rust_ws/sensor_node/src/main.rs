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
use std::time::Duration;
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

        log::info!(
            "Using a client id {} and broker ip {}",
            node_id,
            server_host
        );

        let sensor_fut = sensor_connection::create_sensor();

        // Future that keeps polling until we connect to mqtt
        let client_fut = async {
            loop {
                let res =
                    MqttClient::connect(node_id, &format!("mqtt://{}:1883", &server_host)).await;
                if let Ok(conn) = res {
                    log::info!("Successfully connected to broker");
                    break conn;
                } else if let Err(err) = res {
                    log::error!("Timed out connecting to broker. Err {}", err);
                    // Prevent spam if connect exits early because of lack of IP
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
            }
        };
        join!(sensor_fut, client_fut)
    };
    log::info!("Sensor and mqtt both ready!");

    // Zero sensor
    let mut app = ApplicationContext::new(sensor, 10_000, 200);

    while let Err(err) = app.zero().await {
        log::error!("Failed to zero sensor with: {:?}", err);
    }

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
        let res = client.pub_connected_msg().await;
        if res.is_err() {
            disconnected = true;
            log::error!("Failed connect heartbeat with: {}", res.unwrap_err());
            continue;
        }

        let rcv_fut = client.recv_mqtt_msg();
        let trg_fut = app.wait_for_trigger();
        let timeout = tokio::time::sleep(Duration::from_secs(3));

        // Accept new messages and wait for sensor concurrently (branches are mutually exclusive)
        tokio::select! {
            res = rcv_fut => {
                if res.is_err() {
                    disconnected = true;
                    continue
                }

                let msg = res.unwrap();
                let res = handle_mqtt_msg(msg, &mut client, &mut app).await;
                if let Err(err) = res {
                    match err {
                        error::Error::SensorErr(err) => {
                            if let dist_sensor::SensorError::LunaErr(tf_luna::error::Error::ChecksumFailed) = err {
                                log::error!("Checksum Failed!")
                            }
                        }
                        _ => disconnected = true
                    }
                }
            },
            res = trg_fut => {
                if let Err(err) = res {
                    log::error!("Sensor erred while reading: {}", err);
                    continue
                }
                if handle_trigger(&mut client, &mut app, res.unwrap()).await.is_err() {
                    disconnected = true;
                }
            },
            // Timeout so we send heartbeat to client even if we dont manually zero or detect
            _ = timeout => {
                log::trace!("Timeout to send heartbeat...")
            }
        }
    }
}
