#!/bin/bash

# Runs on the actual sensor node, starting the docker container automagically on SBC boot. BATMAN edition

# Build docker container if not built yet
[ -n "$(docker images -q timebay:sensor)" ] || docker build -t timebay:sensor -f sensor-node.dockerfile .

#TODO set to restart on failure in prod
# Launch sensor node, restarting if it crashed
docker run --privileged --rm --network host -e NODE_ID=1 -e BROKER_HOST=gateway --add-host=gateway:192.168.0.1 --volume /dev:/dev --entrypoint "/sensor_node_bringup_bat.bash" timebay:sensor
