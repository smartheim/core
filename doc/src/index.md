<span style="float:right">
  [![GitHub]][repo]
  [![rustdoc]][docs]
  [![Latest Version]][crates.io]
</span>

[GitHub]: /img/github.svg
[repo]: https://github.com/openhab-nodes/core
[rustdoc]: /img/rustdoc.svg
[docs]: https://github.com/openhab-nodes/core
[Latest Version]: https://img.shields.io/crates/v/ohx-core.svg?style=social
[crates.io]: https://crates.io/crates/ohx-core

<div style="clear:both"></div>

# OHX Developer Guide

Welcome to **The OHX Developer Documentation**, an introductory guide to make you familiar with design principles, the project philosophy, overall architecture as well component specific architectures.

## Who This Guide Is For
OHX follows the idea of micro-services.
You will read about concepts and the overall as well as service individual architecture.

The programming language is [Rust](https://doc.rust-lang.org/book).
This documentation assumes that you’ve written some code in Rust already.

The OHX Core is not the only part that you can contribute to.
Frontend developers as well as App developers are very much welcome.
Head over to the different frontend repositories on http://github.com/openhab-nodes.

## How to Use This Guide

In general, this guide assumes no specific reading order.

The first few chapters outline the design principles and goals of the project.
You probably don't want to miss out on the [Architecture](/developer/architecture) chapter for the bigger picture.

Later chapters are giving inside in some of the design decisions of specific components and how to extend or contribute to those.

## Code of Conduct

We are committed to providing a friendly, safe and welcoming environment for all.
Please check the [Code of Conduct](https://openhabx.com/conduct) on the webpage for more details.

## How to Contribute

OHX uses [Git](https://git-scm.com), especially [GitHub](https://github.com/openhab-nodes) to manage source code, organise issues and project goals.
You will get in contact with Git in many places and it makes sense to get yourself familiar with its basic commands and concepts.
There are many pages to learn about Git.
Try [Git - The Simple Guide](http://rogerdudler.github.io/git-guide) as a start.
In Git it is common to send *Pull Requests* from your own source code clone back to the official repository.

We are always thrilled to receive pull requests, and do our best to process them as fast as possible.
Not sure if that typo is worth a pull request?
Do it! If your pull request is not accepted on the first try, don't be discouraged!
If there's a problem with the implementation, you will receive feedback on what to improve.


## Contribute &amp; Build

Rust 1.39 or newer is required.
Install via [rustup](https://rustup.rs/).<br>
A recommended IDE is [Visual Studio Code](https://code.visualstudio.com/) with the [Rust extension](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust).

On &hellip;

* Linux install [podman](https://github.com/containers/libpod/blob/master/install.md) and [podman-compose](https://github.com/muayyad-alsadi/podman-compose),
* Windows install [Docker Desktop for Windows](https://docs.docker.com/docker-for-windows/install/),
* Mac OS install [Docker Desktop for Mac](https://docs.docker.com/docker-for-mac/install/).

Clone the repository from https://www.github.com/openhab-nodes/core.
Eg `git clone git@github.com:openhab-nodes/core.git` and change to the checked out directory.

Build with `docker-compose -f docker-compose.yml up` (or `podman-compose -f docker-compose.yml up`).

### Non-container

1. Download and extract the newest zip file for your hardware.
   Find it on the [releases page](https://github.com/openhab-nodes/core/releases).
2. Your operating system may prevent the use of "system" ports (ports below 1024).
   If you want port 80 and port 443, add the NET_BINDSERVICE capability on Linux / Mac OS like so:
   `sudo setcap CAP_NET_BIND_SERVICE=+eip ./ohx-serve`
3. Start the `ohx-core`, `ohx-auth`, `ohx-serve`, `ohx-ruleengine` binaries.
   You must start `ohx-core`, all other binaries provide additional features.
   Check the architecture section below to get to know more.
   You might want to start via `sh start.sh` on the command line.
  
Call `ohx-core -h` or any other binary to print a list of command line options to adjust OHX's behaviour.
 Each binary has its own unique command line flags. Check the individual readmes.

> You can start additional Addons without using software containers as well.
   Installing Addons via the *Setup & Maintenance* Web UI is not possible
   and security and resource related restrictions cannot be enforced however.


### Test without Containers

Usually core services are bundled into software containers for distribution, but also for local execution.
Containers are explained in the [Addons](/developer/addons) chapter.

For running a service (or an addon) in a debug session, containers are not recommended.

1.
Follow the steps of the last section except step 5.
Execute `sh ./start_all.sh` on the command line to start OHX without containers.
2.
Stop / kill the service that you want to start in debug mode.
3.
Start the service in question via your IDE or `cargo run` in debug mode.

### Test With Production OHX

It is possible to run a core service on your developer machine and have a (production) OHX installation running on a different system.
Please note that you are leaving safe territory here.

For this to work, you first need to create an access token on the <a class="demolink" href="">Maintenance</a> page with the "ADMIN" and "REMOTE" permissions.

Start your addon with the environment variable `REMOTE_OHX=192.168.1.11` set (change the IP accordingly) and the environment variable `REMOTE_OHX_ACCESS` should be set to your token, like `REMOTE_OHX_ACCESS=e5868ebb4445fc2ad9f9...49956c1cb9ddefa0d421`.

The service should report that it will attempt to connect to a remote machine.

Be aware, that configuration is not shared across devices.

### Core Service Versioning

Core services follow semantic versioning.

The **major version** is increased whenever the gRPC interface got methods removed, arguments changed or fields of a data type changed.
It also increases when environment variables are interpreted differently than before or command line arguments change or get removed.

The **minor version** is increased whenever methods are added to the gRPC interface.
It also increases when environment variables or command line arguments have been added.
A code change that would normally classify as patch version increase, increases the minor version if it is security related.

The **patch version** is increased for documentation changes and code fixes.

## How to develop Addons

Find template repositories on https://github.com/openhab-nodes for different programming languages,
including Rust, NodeJS, Go and C++.

Recommended Addons in Rust for code inspection and learn by example include:

* `hueemulation`: [IO-Service] Registers an http API endpoint on /api and provides the full Hue API, emulating a Hue bridge version 2.
* `hue_deconz`: [Binding] Adds support for Zigbee devices via a hue bridge V2 or (deconz) software hue bridge.
   Shows how to use upnp to find bridges.
* `mqtt_homie`: [Binding+Transformation] Finds registered MQTT Homie devices on a given MQTT Server.
* `mozilla_webthing`: [Binding] Finds Mozilla WebThings in your network. 
* `cloudconnector`: [IO-Service] Amazon Alexa and Google Home support via https://openhabx.com account.
   Uses a super fast, lightweight TCP proxy (see [OHX-Cloud](https://github.com/openhab-nodes/cloud) repository) to provide
   a bridge between the Amazon Alexa servers / Google Home Fulfilment Action service and your local OHX installation.
* `scriptengine_quickjs`: Registers quickjs (ES2020 js engine) for `.js` javascript files.

You deploy your developed Addon to the OHX Addon Registry via the [OHX-Addon-CLI](https://github.com/openhab-nodes/cloud-addon-registry-cli).

### Logging
 
OHX logs are informative logs, no debug outputs.
They help with following what is going on but are not required to maintain an OHX installation and you can happily use
OHX without ever looking at the logs.
- All relevant status data and notifications are accessible on the *Setup & Maintenance* UI and via the gRPC API.
- Telemetry data is feed into InfluxDB (if InfluxDB is running).

Reduce log output with `RUST_LOG=error ohx-core` (standalone) or `docker run ... -e RUST_LOG=error` (docker).
