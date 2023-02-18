# TIMEBAY

TODO

## Setup

### Sensor nodes

Clone timebay. In the timebay directory, run
```shell
scripts/run_sensor_node.bash
```
This will build (if first time) and run the sensor node docker container.
This docker container will handle the connection to the laser sensor as well
as setting up the mesh.

This script is intended to be run on boot of sensor node SBCs.