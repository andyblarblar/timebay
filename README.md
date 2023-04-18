# TIMEBAY

Timebay is a system that provides two services to student vehicles in testing:
1. Sector timing
2. Local network access

It does this by creating a mesh network of sensor nodes, where each sensor node contains a mesh point for other sensor nodes, 
vehicles, and clients to connect to. Timing data passes through this mesh, as does any user applications used, such as video feeds,
ROS topics and more. To connect to the timing services of Timebay, a TUI is provided.

For more details on the design and operations of Timebay, please see [the docs](./docs) and [the paper]().

## Setup and Operation

To setup your own instance of Timebay or maintain an existing one,
please see [this document](docs/le-potato-setup.md).

## Development

### Rust

The sensor node and client are both implemented in Rust. These are both in the /rust_ws folder, in a Rust workspace.

All of these applications can be developed without the nodes themselves, as they do not assume any hardware.

To run the sensor node using a fake instead of a real tf-luna, compile with the `no_sensor` feature enabled. This will
read a trigger every 3 seconds.

To integration test, there is a node simulator in node_sim. This can be used to test edge cases in the GUI or potential future
consumers of the detection data.

## Docker

Both nodes dockerfiles are at the top level. All files used in them are in the top level /scripts, /configs ect. directories.

Most scripts in /scripts have two versions, one for 802.11s, and one for batman-adv. All scripts for batman will have the 
postfix '_bat'. Both BATMAN-adv and 802.11s have their quirks. 802.11s works much better overall, but is pretty strict when it
comes to compatible hardware. BATMAN on the other hand is very compatible, but suffers from huge packet loss.

For convenience, there are build_x_node.bash scripts to build and tag the nodes correctly during development.