---
title: BACnet, IP, and MS/TP
parent: Learn
nav_order: 1
permalink: /learn/bacnet-mstp/
---

# What is BACnet / IP / MS/TP?

This project is a **router**: it forwards BACnet **network-layer** messages
(NPDUs) between two different networks — one on Ethernet/UDP (**BACnet/IP**),
one on a serial RS-485 trunk (**MS/TP**).

## BACnet in one paragraph

BACnet is a building-automation protocol. Devices expose objects (sensors,
setpoints, binary points). Controllers and tools discover each other and read or
write properties. A **router** does not need a full object database of its own;
it moves packets between networks so a tool on IP can talk to a device on MS/TP
(and the other way around).

## BACnet/IP

- Runs over **UDP** (commonly port **47808**).
- Devices have an IP address; BACnet also encodes IP+port as a six-byte “MAC”
  on the wire for routing.
- Easy to lab with normal Ethernet; careful with broadcasts and who binds the
  UDP port (two stacks on the same host/IP often steal each other’s packets).

## MS/TP (Master-Slave/Token-Passing)

- Runs on **RS-485**: a shared two-wire (plus reference) bus.
- Masters pass a **token**; only the station holding the token may initiate
  requests. Timing is tight — milliseconds matter.
- Each station has an 8-bit **MAC** (0–127). `Max_Master` and baud must match the
  trunk you join.
- USB RS-485 adapters show up as Linux serial ports. Always use stable
  `/dev/serial/by-id/...` paths, never a bare `ttyUSB0` in config.

## What this appliance does

```text
BACnet/IP network  ←→  DIY BACnet Router  ←→  MS/TP network
   (UDP / Ethernet)         (Buildroot OS)         (USB RS-485)
```

It uses the pinned [rusty-bacnet](https://github.com/jscott3201/rusty-bacnet)
stack for transport and routing logic, wrapped by this repo’s adapter and
`routerd` management plane.

{: .warning }
Default config keeps **forwarding disabled**. Lab unlock flags (such as
`--route-enable`) are for controlled benches — not for a random production trunk
without a plan.
