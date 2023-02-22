#!/bin/bash

# Script that runs on boot of the sensor node docker container

ifacename="wlxec086b180ec6" # TODO make this accurate for Pis

# Setup mesh, ip will be set by dhcpcd
ip link set down $ifacename
iw dev $ifacename set type mesh
iw dev $ifacename set meshid timebay
iw dev $ifacename set channel 1 HT40+
ip link set up $ifacename

# Set default route to gateway node
ip route add default 192.168.0.1/24 dev $ifacename

sensor_node