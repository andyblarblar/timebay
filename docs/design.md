# Design

This document has some notes on the design of the system

- All mqtt use should be generic over client implementation. This means that we can swap in the case of issues with
cross compiling ect.
- Dist sensors are debounced to avoid triggering on the same vehicle multiple times
- Mqtt payloads are postcard serialized structs