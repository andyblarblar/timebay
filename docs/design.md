# Design

This document has some notes on the design of the system

## Sensor node choices
- All mqtt use should be generic over client implementation. This means that we can swap in the case of issues with
cross compiling ect.
- Dist sensors are debounced to avoid triggering on the same vehicle multiple times
- Mqtt payloads are postcard serialized structs
- Sensor threshold should always be > 160mm, since the sensor has +-16cm accuracy at worst
  - Note the 20cm blind spot, although this shouldn't matter much because the vehicle should never be that close anyhow.

## Networking choices
- Choice of layer 1:
  - 802.11 was chosen for its ability to maintain high throughput, for things like FPV driving