mod application;
mod dist_sensor;
mod error;
mod handlers;
mod mqtt;
mod sensor_connection;

use crate::application::ApplicationContext;
use crate::handlers::{handle_mqtt_msg, handle_trigger};
use crate::mqtt::MqttClient;
use std::time::Duration;
use tokio::join;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_log::console("debug").unwrap();

    log::info!("Waiting for mqtt and sensor to connect...");

    let (sensor, mut client) = {
        let node_id = 1; //TODO get from env var

        let sensor_fut = sensor_connection::create_sensor();

        // Future that keeps polling until we connect to mqtt
        let client_fut = async {
            loop {
                if let Ok(conn) = MqttClient::connect(node_id, "mqtt://localhost:1883").await {
                    //TODO set real broker ID
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

    loop {
        let rcv_fut = client.recv_mqtt_msg();
        let trg_fut = app.wait_for_trigger();

        // Accept new messages and wait for sensor concurrently (branches are mutually exclusive)
        tokio::select! {
            res = rcv_fut => {
                let msg = res?; //TODO make sure the docker container has auto-bringup so these unwraps are recoverable
                handle_mqtt_msg(msg, &mut client, &mut app).await?;
            },
            res = trg_fut => {
                let dist = res?;
                handle_trigger(&mut client, &mut app, dist).await?;
            }
        }
    }

    Ok(())
}
