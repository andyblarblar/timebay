#!/bin/bash

# Runs on the actual sensor node, starting the docker container automagically on SBC boot

# Build docker container if not built yet
[ -n "$(docker images -q timebay:sensor)" ] || docker build -t timebay:sensor -f sensor-node.dockerfile .

# Launch sensor node, restarting if it crashed
docker run --privileged --restart on-failure --network host -e NODE_ID=1 -e BROKER_HOST=gateway --add-host=gateway:192.168.0.1 --volume /dev:/dev timebay:sensor