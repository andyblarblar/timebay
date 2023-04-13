use crate::cli::MENU_DIALOG;
use clap::Parser;
use std::collections::HashMap;
use std::io::stdin;
use std::time::{SystemTime, UNIX_EPOCH};
use timebay_common::messages::MqttMessage::{Connection, Detection, Disconnection};
use timebay_common::messages::{ConnectionMessage, DetectionMessage, DisconnectionMessage};
use timebay_common::mqttclient::MqttClient;

mod cli;

// This code is not good lol
#[tokio::main]
async fn main() {
    let cli = cli::Args::parse();
    let mut clients = HashMap::new();

    // Connect a client for each node
    for node_id in cli.node_ids {
        let mqtt = MqttClient::connect(
            &format!("mqtt://{}:1883", cli.broker_host),
            &format!("node{}", node_id),
            &[],
            None,
        )
        .await
        .unwrap();

        clients.insert(node_id, mqtt);
        println!("Connected node {}", node_id);
    }
    println!("All nodes connected.");
    println!("Welcome to node sim!");

    loop {
        let user_in = input(MENU_DIALOG);

        match user_in.trim() {
            "1" => {
                let node = input("Which node?\n");

                if let Ok(nodeid) = node.trim().parse::<u16>() {
                    let fut = clients
                        .get(&nodeid)
                        .map(|client| client.publish(Connection(ConnectionMessage::new(nodeid))));

                    if let Some(fut) = fut {
                        fut.await.unwrap();
                    } else {
                        println!("Node did not exist, or broker is not connected.");
                    }
                } else {
                    println!("Could not parse node.");
                }
            }
            "2" => {
                let node = input("Which node?\n");

                if let Ok(nodeid) = node.trim().parse::<u16>() {
                    let fut = clients.get(&nodeid).map(|client| {
                        client.publish(Disconnection(DisconnectionMessage::new(nodeid)))
                    });

                    if let Some(fut) = fut {
                        fut.await.unwrap();
                    } else {
                        println!("Node did not exist, or broker is not connected.");
                    }
                } else {
                    println!("Could not parse node.");
                }
            }
            "3" => {
                let node = input("Which node?\n");

                if let Ok(nodeid) = node.trim().parse::<u16>() {
                    let dist = input("Distance(mm)?\n");
                    if let Ok(dist) = dist.trim().parse::<u32>() {
                        let fut = clients.get(&nodeid).map(|client| {
                            let now = SystemTime::now();
                            client.publish(Detection(DetectionMessage::new(
                                nodeid,
                                dist,
                                now.duration_since(UNIX_EPOCH).unwrap().as_secs(),
                                now.duration_since(UNIX_EPOCH).unwrap().subsec_nanos(),
                            )))
                        });

                        if let Some(fut) = fut {
                            fut.await.unwrap();
                        } else {
                            println!("Node did not exist, or broker is not connected.");
                        }
                    } else {
                        println!("Could not parse distance.");
                    }
                } else {
                    println!("Could not parse node.");
                }
            }
            _ => println!("Invalid input."),
        }
    }
}

/// Python input function
fn input(prompt: &str) -> String {
    print!("{}", prompt);
    let mut str = String::new();
    stdin().read_line(&mut str).unwrap();
    str
}
