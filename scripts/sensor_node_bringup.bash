#!/bin/bash

# Script that runs on boot of the sensor node docker container

ifacename="wlxec086b180ec6" # TODO make this accurate for Pis

# Setup mesh
ip link set down $ifacename
iw dev $ifacename set type mesh
iw dev $ifacename set meshid timebay
ip link set up $ifacename

# TODO add static ips or dhcp here

sensor_node