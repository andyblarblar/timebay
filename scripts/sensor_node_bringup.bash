#!/bin/bash

# Script that runs on boot of the sensor node docker container

# Find interface with mesh support
device="$(python3 find_mesh_interface.py)"

# Setup mesh, ip will be set by dhcpcd
ip link set down "$device"
iw dev "$device" set type mesh
iw dev "$device" set meshid timebay
iw dev "$device" set channel 5 HT40+
ip link set up "$device"

# Get NTP time from gateway
chronyd

# Request an IP, will run in background as well
dhcpcd -4 --noipv4ll --allowinterfaces "$device"

sensor_node
