mod sensor_connection;
mod types;
use paho_mqtt::{ConnectOptionsBuilder, CreateOptionsBuilder, Message};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_log::console("debug").unwrap();

    let sensor = sensor_connection::create_sensor().await;

    let client = CreateOptionsBuilder::new()
        .client_id("node1") //TODO Use env var
        .server_uri("tcp://localhost:1338")
        .create_client()?;

    // Set LWT
    let mut conn_opt = ConnectOptionsBuilder::default();
    let lwt = Message::new("/disconnect", [1], 2); //TODO change payload to node #
    conn_opt.will_message(lwt);

    client.connect(conn_opt.finalize()).await?;

    //TODO pub connection and enter sensor loop

    Ok(())
}
