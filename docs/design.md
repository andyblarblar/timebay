# Design

This document has some notes on the design of the system

## Sensor node choices
- All mqtt use should be generic over client implementation. This means that we can swap in the case of issues with
cross compiling ect.
- Dist sensors are debounced to avoid triggering on the same vehicle multiple times
- Mqtt payloads are postcard serialized structs
- Sensor threshold should always be > 160mm, since the sensor has +-16cm accuracy at worst
  - Note the 20cm blind spot, although this shouldn't matter much because the vehicle should never be that close anyhow.
- The dockerfile should be cross compilable to other archs, like MIPS
- While the TF-Luna does have an object detection mode that more or less eliminates the need for the dist sensor, I use
the polling mode anyway, so we can swap to something like an ultrasonic sensor in the future with no code changes.

## Networking choices
- Choice of layer 1:
  - 802.11 was chosen for its ability to maintain high throughput, for things like FPV driving
- Mesh interface config is done in the docker container, so that it can be run on any linux system without the need
to worry about configuring it beforehand.
  - This is possible by using network=host
- While 802.11s was initially chosen, it appears that the ability to bridge mesh interfaces was never actually implemented
, despite being in the docs. [see](https://www.spinics.net/lists/linux-wireless/msg19548.html).
  - This manifested as ARP frames being transmitted across the mesh, but not to the bridge (verified with wireshark). This prevented eth0 from 
  being connected to the mesh, as well as DHCP from being served.
  - 80211s on linux also seems pretty abandoned, with poor driver support (ex. rpi)
- BATMAN-adv over IBSS is added as an alternative to 80211s as the meshing layer for timebay
  - Running over an IBSS means pretty much any interface can be used, although it does lock us into linux (so no MCU)
  - It also puts [an emphasis](https://www.open-mesh.org/projects/batman-adv/wiki/Wiki) on being able to be bridged, so it should solve our 80211s woes there
    - In particular, it's actually interface agnostic, and can even run over bluetooth and ethernet!
  - This is done by just changing the docker entrypoint to a different script, so we don't need duplicate containers

