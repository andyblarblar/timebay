#!/bin/bash

# Runs on the actual external node, starting the docker container automagically on SBC boot.

# Change to timebay directory if running in cronjob
cd "$(dirname "$0")/..";

git fetch

# Update if out of date
if [ $(git rev-parse HEAD) != $(git rev-parse @{u}) ]; then
  # We must have network connection to get here
  git pull
  # This will always rebuild the docker container
  docker build -t timebay:ext -f external-node.dockerfile .
fi

# Build docker container if not built yet
[ -n "$(docker images -q timebay:ext)" ] || docker build -t timebay:ext -f external-node.dockerfile .

# Launch gateway node
docker run --privileged --rm -t -d --network host --cap-add SYS_TIME --add-host=gateway:192.168.0.1 --entrypoint "/external_node_bringup.bash" timebay:ext
