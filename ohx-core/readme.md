# OHX Core

## Command Line Options

The **OHX Root directory** is by default the working directory.
You may change this by starting with `ohx-core -r your_directory`.
By default OHX-Core will create the OHX root directory structure including
`backups`, `certs`, `config`, `scripts`, `rules` and `webui`, if it not yet exists.

If you provide an https certificate (x509 in *der* format) via `OHX-ROOT/certs/key.der` and `OHX-ROOT/certs/cert.der`,
OHX Core will use it.
If no certificates exist, OHX will create a self-signed one valid for one year which is refreshed 14 days before expiry.

**Container service:** OHX will auto-detect if "docker" or "podman" should be used for container management. "podman" is preferred".
If you want to use "docker" instead, execute with `ohx-core --force-docker`.

OHX-OS applies a filesystem based **quota for persistent storage** per Addon directory.
It is out of scope for OHX to ensure that limit on a standalone installation accurately.
A best effort approach is used (by watching directories and checking file sizes once in a while),
which simply stops an Addon that exceeds the quota. You can disable this via `ohx-core --disable-quota-enforcement`.

**Self healing**: OHX Core tries its best to keep running and cope with certain conditions
like expired certificates, low disk space and low memory.
Set policies for each condition, for example `ohx-core --low-memory-policy=gradually-restart-addons --low-disk-space-policy=stop-addons`.
