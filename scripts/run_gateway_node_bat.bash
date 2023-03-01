#!/bin/bash

# Runs on the actual gateway node, starting the docker container automagically on SBC boot. BATMAN edition

# Build docker container if not built yet
[ -n "$(docker images -q timebay:gate)" ] || docker build -t timebay:gate -f gateway-node.dockerfile .

# Launch gateway node
docker run --privileged --restart on-failure -t -d --network host --entrypoint "/gateway_node_bringup_bat.bash" timebay:gate
