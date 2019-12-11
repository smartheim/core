# Addons

This directory contains the Yaml / Docker compose based Addon description files of installed Addons. Those are used on OHX startup to start the Addon software containers.

After a backup restore, those are used to reinstall missing Addons.

## Pre-provisioning

You may have .tar.xz compressed docker "load" compatible images in this directory as well.
The "full" backup procedure will generate those files.
Such a file must be named exactly like the corresponding addon description file, apart from the file extension. For example:

* "ohx-addon-mqtt-homie.yaml"
* "ohx-addon-mqtt-homie.tar.xz"

**Security note**: A `checksums.txt` file must be present with key=value entries where filename=sha256.
Only those image files are considered that match the noted checksum.
The last entry must be `signed=signed_jwt_token` where *signed_jwt_token* is a signature string (a JWT) that is used to verify the list itself. Please note that an installation specific key (found in the `certs` directory) is used to generate and verify the signature.

NEVER share the certs directory. If you plan to hand the directory / installation over to a friend with pre-provisioned addons: First delete the certs directory. Start ohx to generate new certificates.

You can generate the `checksums.txt` file with the OHX CLI: `ohx-cli addons signature`. 

If you want to prepare an Addon directory with your currently installed Addons use:
`ohx-cli addons to_directory`. This will also generate the `checksums.txt` file.

If you want to add an Addon directly from the registry without installing it first, use:
`ohx-cli addon_dir add the_addon` with *the_addon* being the id of the addon.