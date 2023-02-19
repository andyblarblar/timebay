# Network

TODO go over the overall architecture

## Mesh

An 802.11s mesh network is created between all the sensor nodes and the gateway node. This network does not use encryption,
since that is not required for our use case and will add extra lag to hand-offs.

## IP

To provide TCP/IP networking over the mesh (required for MQTT and clients), the gateway node will also host a DHCP server.
All sensor nodes will use this DHCP server for their IPs, while the gateway node will have the static IP 192.168.0.1 on the 
mesh interface, and 192.168.0.2 on its bridge respectively.

Each of the sensor nodes has the record "gateway:192.168.0.1" in their hosts file, to allow for connecting to the MQTT
broker using hostnames rather than IP.

## Bridge

The gateway node bridges its mesh and ethernet interfaces to allow for the end client to both connect to the MQTT broker 
and nodes in the mesh. This could be used for example to connect to a camera on the car from a laptop, to facilitate an
FPV setup.

## Time

TODO go over NTP once we see if it works
