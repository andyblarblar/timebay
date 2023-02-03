mod application;
mod dist_sensor;
mod error;
mod mqtt;
mod sensor_connection;

use crate::application::ApplicationContext;
use crate::mqtt::MqttClient;
use std::time::Duration;
use tokio::join;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_log::console("debug").unwrap();

    let node_id = 0; //TODO get from env var

    let sensor_fut = sensor_connection::create_sensor();

    // Future that keeps polling until we connect to mqtt
    let client_fut = async {
        loop {
            if let Ok(conn) = MqttClient::connect(node_id, "tcp://localhost:1338").await {
                log::info!("Successfully connected to broker");
                break conn;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    log::info!("Waiting for mqtt and sensor to connect...");
    let (sensor, client) = join!(sensor_fut, client_fut);
    log::info!("Sensor and mqtt both ready!");

    // Zero sensor
    let mut app = ApplicationContext::new(sensor, 10);
    log::debug!("Set zero to {}mm", app.zero().await);

    //TODO enter sensor loop

    Ok(())
}
