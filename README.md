# TIMEBAY

TODO

## Setup

### Sensor nodes

First, connect the tf-luna to the SBC via an adapter or GPIO. Also connect a mesh compatible Wi-Fi adapter, if the SBCs
built in Wi-Fi is does not support mesh-point in its options (only if using 802.11s).

Next, install docker if it is not already.

Next, connect the SBC to a network and clone timebay. In the timebay directory, run

For 802.11s:

```shell
scripts/run_sensor_node.bash
```

For BATMAN-adv over IBSS (recommended):

```shell
scripts/run_sensor_node_bat.bash
```

For
This will build (if first time) and run the sensor node docker container.
This docker container will handle the connection to the laser sensor as well
as setting up the mesh.

This script is intended to be run on boot of sensor node SBCs, ex. via crontab.

### Gateway node

First, ensure a mesh compatible Wi-Fi interface is available (only if using 802.11s), and that docker is installed.

Ensure that the ethernet interface, the mesh interface (you can find this by running scripts/find_mesh_interface.py),
and bat0 are all denied in the dhcpcd config.

Clone timebay. In the timebay directory, run

For 802.11s:

```shell
scripts/run_gateway_node.bash
```

For BATMAN-adv over IBSS (recommended):

```shell
scripts/run_gateway_node_bat.bash
```

This will build (if first time) and run the gateway node docker container.
This docker container will handle setting up the mesh, the bridge to ethernet, and
the mosquitto and dhcp daemons.

This script is intended to be run on boot of the gateway node SBC, ex. via crontab.