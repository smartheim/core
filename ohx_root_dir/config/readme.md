# Configurations

Each Addon gets its own directory in here.
It is volume mounted into the software container, an Addon will therefore only see its own files.

On OHX-OS each Addon directory is quota limited to 10, 50, 100, 1000 MB depending on the Addons permissions.

OHX services use "ohx-core", "ohx-auth" and "ohx-ruleengine" directories.

## Content

All .json files in the respective subdirectory are handled as configuration files.
For example "config/ohx-addon-homie/schema_id.my-device.json".

A file name in this directory must adhere to the pattern "schema_id.config_id.json".

* An Addon must provide a json schema (and optionally a json ui schema) per referenced "schema_id"
via the programming language specific OHX API.
* The schema can dynamically adapt to the Addons current state and libohx will propagate changes
to the core which informs connected Web UIs.
* Some Addons might allow the user to create multiple configuration files for a given "schema_id". 

The "Setup & Maintenance" Web UI will use configuration files together with json (ui) schemas
to present a user interface for changing and displaying those configuration.
And for creating new configurations for a given schema_id if the Addons allows this.

## Other data

Addons and OHX Core may use subdirectories to store other data than json configuration,
for example cached data or internal configuration. For example:
"config/ohx-addon-homie/mqtt_servers/127.0.0.1_fingerprint.txt"