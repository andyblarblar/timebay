#!/bin/bash

# Runs on the actual sensor node, starting the docker container automagically on SBC boot. BATMAN edition
# Node id is passed via arg

# Change to timebay directory if running in cronjob
cd "$(dirname "$0")/..";

git fetch

# Update if out of date
if [ $(git rev-parse HEAD) != $(git rev-parse @{u}) ]; then
  # We must have network connection to get here
  git pull
  # This will always rebuild the docker container
  docker build -t timebay:sensor -f sensor-node.dockerfile .
fi

# Build docker container if not built yet
[ -n "$(docker images -q timebay:sensor)" ] || docker build -t timebay:sensor -f sensor-node.dockerfile .

if [ -z "$1" ]
 then
     echo "No node id passed!"
     exit 1
fi

# Launch sensor node, restarting if it crashed
docker run --privileged --rm --network host -e NODE_ID="$1" -e BROKER_HOST=gateway --add-host=gateway:192.168.0.1 --cap-add SYS_TIME --volume /dev:/dev --entrypoint "/sensor_node_bringup.bash" timebay:sensor
