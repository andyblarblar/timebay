# Design

This document has some notes on the design of the system

## Sensor node choices

- All mqtt use should be generic over client implementation. This means that we can swap in the case of issues with
  cross compiling ect.
- Dist sensors are debounced to avoid triggering on the same vehicle multiple times
- Mqtt payloads are postcard serialized structs
- Sensor threshold should always be > 160mm, since the sensor has +-16cm accuracy at worst
    - Note the 20cm blind spot, although this shouldn't matter much because the vehicle should never be that close
      anyhow.
- The dockerfile should be cross compilable to other archs, like MIPS
- While the TF-Luna does have an object detection mode that more or less eliminates the need for the dist sensor, I use
  the polling mode anyway, so we can swap to something like an ultrasonic sensor in the future with no code changes.
- Connected messages are sent continuously as a heartbeat, since when they are sent only once a late connecting GUI
  cannot
  discover the node.

## Networking choices

- Choice of layer 1:
    - 802.11 was chosen for its ability to maintain high throughput, for things like FPV driving
- Mesh interface config is done in the docker container, so that it can be run on any linux system without the need
  to worry about configuring it beforehand.
    - This is possible by using network=host
- While 802.11s was initially chosen, it appears that the ability to bridge mesh interfaces was never actually
  implemented
  , despite being in the docs. [see](https://www.spinics.net/lists/linux-wireless/msg19548.html).
    - This manifested as ARP frames being transmitted across the mesh, but not to the bridge (verified with wireshark).
      This prevented eth0 from
      being connected to the mesh, as well as DHCP from being served.
    - 80211s on linux also seems pretty abandoned, with poor driver support (ex. rpi)
- BATMAN-adv over IBSS is added as an alternative to 80211s as the meshing layer for timebay
    - Running over an IBSS means pretty much any interface can be used, although it does lock us into linux (so no MCU)
    - It also puts [an emphasis](https://www.open-mesh.org/projects/batman-adv/wiki/Wiki) on being able to be bridged,
      so it should solve our 80211s woes there
        - In particular, it's actually interface agnostic, and can even run over bluetooth and ethernet!
    - This is done by just changing the docker entrypoint to a different script, so we don't need duplicate containers
- In order to avoid latency in the system effecting lap times, detection messages will be timestamped globally when the
  detection occurs on device. This relies on
  all nodes having a synchronised clock.

## Client choices

- Client can survive disconnects
    - The current splits however will not, although this shouldn't be an issue since the client will be hardwired
- System can run with 1-2^16 nodes
    - Really only matters that we support the single sensor node case
- Sector notes:
    - Sector are defined by a starting and ending node, where the ending node is the next sector starting node
        - The last segment is an exception, as its end node is the first node.
    - Timing starts when first node is passed
        - Will not start if any other node is passed
    - If this is not the first lap, then immediately start the next lap when the last lap ends
        - This is needed as the last node of the last lap and the first node of this lap are the same, so we need count
          it as a trigger for both
    - Sectors are completed when their end node is triggered
    - If we jump a node, then mark any sector starting at the current segment and up to the segment ending with the node
      triggered as invalid.
        - We keep timing however, so we still get a final result
    - If we go back a node, just ignore (likely someone just walking on course)
    - Node connection, disconnections, and reconnections are implicitly handled
        - New nodes are ignored
        - A node disconnecting will be handled by the following node being seen as a jump, invalidating the disconnected
          nodes sector
        - Because of this a node can disconnect and reconnect in the same run transparently so long as it triggers when
          it should
    - A run is done when the last non-invalidated segment is complete
- Sector times are calculated by the difference in time between the last completed sector and the next completed sector.
    - This means that if a sector is invalidated, the time spent in that sector will be rolled into the next.
- The client is designed such that it can be started before any other node on a cold start, and passively manage itself
  as the user connects the sensor nodes.
    - This should speed up setup, since one can visually determine if a node is connected by checking the clients screen
      from a distance.
- Client architecture:
    - The client uses a custom Elm like architecture, implemented on cursive, a TUI library.
    - The idea is basically that the gui holds no state, and will be completely redrawn when the backend updates.
        - The backend can only change state on reaction to messages, which are sent from the gui and from external
          sources, like MQTT.
    - The benefit of this system is that the gui will always match the state of the backend, which means that if we unit
      test the backend, that will implicitly also test the gui.
        - It also handles concurrency well, IMO
    - This is actually implemented by message passing between threads
      - The gui runs on the main thread, the backend runs on the backend thread, and async events (MQTT, gui updates ect)
      are processed on an eventloop thread.
      - The gui inputs have callbacks that send messages to the async thread
      - The async thread listens to mqtt and gui messages via polling, and merges the various sources to a common message
      type sent to the backend thread
    - When the backend thread receives a message, it passes it to an update function, which performs side effects on the apps state
      - Update can also return async tasks to run on the async thread, if we have a long-running side effect like sending a mqtt message
    - After running update, the whole gui is popped, and a new gui is rendered and pushed.