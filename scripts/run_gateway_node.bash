#!/bin/bash

# Runs on the actual gateway node, starting the docker container automagically on SBC boot

# Build docker container if not built yet
[ -n "$(docker images -q timebay:gate)" ] || docker build -t timebay:gate -f gateway-node.dockerfile .

# Launch gateway node, restarting if it crashed
docker run --privileged --restart on-failure --network host timebay:gate