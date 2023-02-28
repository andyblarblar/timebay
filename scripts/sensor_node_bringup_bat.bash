#!/bin/bash

# Script that runs on boot of the sensor node docker container, batman edition

# Find interface with adhoc support
device="$(python3 find_adhoc_interface.py)"

# Setup IBSS, dhcp will set IP
ip link set down "$device"
iw dev "$device" set type ibss
ip link set up mtu 1532 dev "$device"
iw dev "$device" ibss join timebay 2412 HT40+

# Setup BATMAN
batctl if add "$device"
ip l set bat0 up

sensor_node
