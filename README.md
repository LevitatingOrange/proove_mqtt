# Proove MQTT

This is a bridge program, to be run on a raspberry pi. It will 
accept mqtt messages from the specified broker and send 
commands to cheap those radio controlled sockets that speak the 
Proove protocol. Read the `examples/example_config.toml` for details.
If you start out fresh, we advise you to disable compatibility mode in 
the config and assign a unique id for each socket you have, grouping 
them however you choose. Then set each of them to reprogram mode (often 
by pressing the button for a longer time) and send a on command via mqtt.

## MQTT topics

The device with the name `<device name>` in the group with the name `<group name>` 
will be settable by sending an MQTT Packet with the payload "on" or "off" and 
the topic `/<root_topic>/set/<group name>/<device name>` to the specified 
broker.