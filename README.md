# TIMEBAY

TODO

## Setup

> Note: for many boards, there is a bit more setup to be done. This depends on the OS being run. For example
> ubuntu machines need to remove a networkd config for adhoc networks. Instructions for boards can be found in /docs.

### Sensor nodes

First, connect the tf-luna to the SBC via an adapter or GPIO. Also connect a mesh compatible Wi-Fi adapter, if the SBCs
does not have built in Wi-Fi.

Next, install docker if it is not already.

Next, connect the SBC to a network and clone timebay. In the timebay directory, run

For BATMAN-adv over IBSS:

```shell
scripts/run_sensor_node_bat.bash <NODE ID>
```

For
This will build (if first time) and run the sensor node docker container.
This docker container will handle the connection to the laser sensor as well
as setting up the mesh.

This script is intended to be run on boot of sensor node SBCs, ex. via crontab.

### Gateway node

First, ensure an adhoc compatible Wi-Fi interface is available, and that docker is installed.

Also ensure bridging iptables are disabled via the kernel parameters:
```
net.bridge.bridge-nf-call-iptables=0
net.bridge.bridge-nf-call-ip6tables=0
net.bridge.bridge-nf-call-arptables=0
```

Clone timebay. In the timebay directory, run

For BATMAN-adv over IBSS:

```shell
scripts/run_gateway_node_bat.bash
```

This will build (if first time) and run the gateway node docker container.
This docker container will handle setting up the mesh, the bridge to ethernet, and
the mosquitto and dhcp daemons.

This script is intended to be run on boot of the gateway node SBC, ex. via crontab.

TODO add client

## Development

### Rust

The sensor node and client are both implemented in Rust. These are both in the /rust_ws folder, in a Rust workspace.

All of these applications can be developed without the nodes themselves, as they do not assume any hardware.

To run the sensor node using a fake instead of a real tf-luna, compile with the `no_sensor` feature enabled. This will
read a trigger every 3 seconds.

To integration test, you can just run the sensor node using its docker container and connect with the client on localhost,
so long as you have a local broker running.

## Docker

Both nodes dockerfiles are at the top level. All files used in them are in the top level /scripts, /configs ect. directories.

Most scripts in /scripts have two versions, one for 802.11s, and one for batman-adv. All scripts for batman will have the 
postfix '_bat'. For reasons elaborated in the paper, 802.11s is not a viable protocol as is for Timebay. Its implementation is 
left for reference, but is not intended for use.

For convenience, there are build_x_node.bash scripts to build and tag the nodes correctly during development.