# Configurations

Each Addon gets its own directory in here.
It is volume mounted into the software container, an Addon will therefore only see its own files.

On OHX-OS each Addon directory is quota limited to 10, 50, 100, 1000 MB depending on the Addons permissions.

OHX services use "ohx-core", "ohx-auth" and "ohx-ruleengine" directories.

## Content

All .json files in the respective subdirectory are handled as configuration files.
For example "config/ohx-addon-homie/my-device.json".

An Addon should(!) provide a json schema (and json ui schema if necessary) per configuration file
via the programming language specific OHX API.

The "Setup & Maintenance" Web UI will use configuration files together with json (ui) schemas
to present a user interface for changing and displaying those configuration.

## Other data

Addons and OHX Core may use subdirectories to store other data than json configuration,
for example cached data or internal configuration. For example:
"config/ohx-addon-homie/mqtt_servers/127.0.0.1_fingerprint.txt"