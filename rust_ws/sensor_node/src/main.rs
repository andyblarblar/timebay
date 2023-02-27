mod application;
mod dist_sensor;
mod error;
mod handlers;
mod mqtt;
mod sensor_connection;

use crate::application::ApplicationContext;
use crate::handlers::{handle_mqtt_msg, handle_trigger};
use crate::mqtt::MqttClient;
use log::LevelFilter::Debug;
use simplelog::{ColorChoice, CombinedLogger, TerminalMode};
use std::time::Duration;
use tokio::join;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    CombinedLogger::init(vec![simplelog::TermLogger::new(
        Debug,
        simplelog::ConfigBuilder::default().build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    log::info!("Waiting for mqtt and sensor to connect...");

    let (sensor, mut client) = {
        // Get broker and node id from env vars
        let node_id = std::env::var("NODE_ID")
            .unwrap_or_else(|_| "1".to_string())
            .parse()?;
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
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        };
        join!(sensor_fut, client_fut)
    };
    log::info!("Sensor and mqtt both ready!");

    // Zero sensor
    let mut app = ApplicationContext::new(sensor, 10_000, 1000);

    let mut disconnected = false;
    loop {
        // Attempt reconnect on disconnect
        if disconnected {
            log::error!("Disconnected from broker, attempting reconnect...");
            while let Err(err) = client.reconnect().await {
                log::error!("Erred with {} during reconnect attempt!", err);
            }
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
                handle_mqtt_msg(msg, &mut client, &mut app).await?;
            },
            res = trg_fut => {
                let dist = res?;
                handle_trigger(&mut client, &mut app, dist).await?;
            }
        }
    }
}
