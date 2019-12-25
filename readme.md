# OHX Core

[![Build Status](https://github.com/openhab-nodes/core/workflows/test/badge.svg)](https://github.com/openhab-nodes/core/actions)
[![](https://img.shields.io/badge/license-MIT-blue.svg)](http://opensource.org/licenses/MIT)

> OHX is a modern Smart Home solution, embracing technologies like software containers for language agnostic extensibility. Written in Rust with an extensive test suite, OHX is fast, efficient, secure and fun to work on.

OHX Core consists of multiple services that work in tandem to form a vendor crossing **device interconnect hub** 
(ie connect a ZWave wall switch with a Zigbee light bulb) and Smart Home solution.
Find the individual projects in their respective subdirectories.
You usually want to use additional OHX Addons for specific device support.

For a ready to use operating system image for single board computers and regular PCs (and virtual machines),
you might also be interested in [OHX OS](https://github.com/openhab-nodes/ohx-os/).

> OHX Core Services are safe to be exposed to the Internet and implement encryption, authentication various anti-abuse techniques
like IP rate limiting, limited backpressure queues, limited input buffers and safe packet parsing.


!!! WARNING !!!

This is not yet an [MVP](https://en.wikipedia.org/wiki/Minimum_viable_product).
Only certain parts work and will break again during development.

## Table of Contents

1. [Download and Start up](#download-and-start-up)
1. [Usage](#usage)
1. [OHX Services](#ohx-services)
1. [Compile and Contribute](#compile-and-contribute)
1. [Security](#security)
	1. [Report a security event or vulnerability](#report-a-security-event-or-vulnerability)
1. [Maintainer: Core Deployment](#maintainer-core-deployment)

## Download and Start up

Please note that OHX can be run as standalone binaries.
This option is helpful for development and debugging and explained in the [OHX Developer Documentation book](https://github.com/openhab-nodes/core).

This section however explains how to run OHX via Docker containers.

> **About software containers**: A software container can easily and in a standardized way be restricted in its resource usage
(Memory, CPU, File Access, Network Access, limited Kernel API) to protect the host system from potential malicious Addons or just badly written Addons.

**Prerequirement:** You must have [Docker](https://www.docker.com/products/docker-desktop) or a Docker compatible (for example [`podman`](https://podman.io/getting-started/installation)) command line tool installed.

Use the `docker-compose.yml` file to start all relevant containers.
Alternatively use the `start_containers.sh` file if you do not have access to docker compose.

- By default OHX core services will try to start on http port 8080 and https port 8443.
  Docker port routing is used to expose OHX on port 80 and 443.
- OHX core service containers require a few mounted directories.
  The mentioned start up methods will use a "ohx_root_dir" directory within the current working directory.
- Use  `docker run -rm -it docker.pkg.github.com/openhab-nodes/core/ohx-core -h` to print a list of command line options to adjust OHX's behaviour.

## Usage

The *Setup & Maintenance* UI can be found on `https://localhost/`, if you are running OHX on the same system,
and on `https://ohx.local/` if you are using [OHX OS](https://github.com/openhab-nodes/ohx-os/).
The pre-configured password for the administrative user is "ohxsmarthome".

You may also use the command line tool (find it in the *releases* zip file) `ohx-cli` to interact with OHX.
Use `ohx-cli --help` to print all available commands and `ohx-cli the_command --help` to show command specific help.

Usually you first want to detect running OHX instances by calling `ohx-cli detect` and than select one of the found
instances to be used for further calls: `ohx-cli login 192.168.1.17:443`.

## OHX Services

In-depth explanations are given in the developer documentation.
A quick run down on individual service responsibilities follows.

`ohx-core` is a thin supervisor for software containers (it uses the `docker` or `podman` CLI interface internally)
  to install, start and manage OHX Addons and access Addon logs.
- Core provides the interconnection service.
  This service reacts on Addon Thing property changes and sends corresponding *Commands* to Addons.
- IOServices are provided with changed Addon Thing property, if the configured filters pass and
  received *Commands* are routed to Addons if, again, the configured filters pass.
- ohx-core acts as a notification service. Addons can extend the service by providing additional notification channels.
- Backup strategies are executed by core. 

`ohx-serve` is a static https file server for web-uis.
- It generates a self-signed https certificate if none is found at start up and redirects http requests to https.
- It provides a REST-like access (GET/POST/PUT/DELETE) to ioservice and interconnection configurations, rules, scripts, and general configuration. 
- Without `ohx-serve` there will be no http(s) server, so no *Setup & Maintenance* Web UI and no way to
 manipulate configurations, rules, scripts via a web API. You can still just alter the file files for configuration.

`ohx-ruleengine` is an [Event, Condition, Action](https://en.wikipedia.org/wiki/Event_condition_action) rule engine.
- Addons can register additional "Events", "Conditions", "Actions" and "Transformations" types.
 Please check the Rust generated documentation as well as the more detailed rule engine [readme](ruleengine/readme.md).
- Without `ohx-ruleengine` scripts and rules are not enabled, but Addon interconnection does work.
If you only require the interconnect functionality, just do not start up `ohx-ruleengine`.

`ohx-auth` is an Identity and Access Management service, based on OAuth with JWT tokens and OHX specific scopes.
- User accounts are stored in flat files.
- It also manages extern OAuth Tokens and token refreshing, like the https://openhabx.com cloud link for
Amazon Alexa and Google Home support.
- Without `ohx-auth` you will not be able to login via the command line utility or the *Setup & Maintenance* Web UI.
 Some Addons require periodic token refreshing. Those Addons will not work.
 
## Compile and Contribute

OHX is written in [Rust](https://rustup.rs/).
You can develop for Rust in Jetbrains CLion, Visual Studio Code, Visual Studio and Eclipse.
Compile with `cargo build` and for production binaries use `cargo build --release`.

Run with `./build_and_start.sh`.

PRs are welcome.
* A PR is expected to be under the same license as the repository itself and must pass the
test suite which includes being formatted with `rustfmt`.
* Newly introduced dependencies must be under any of the following licenses: MIT, Apache 2, BSD.
* OHX follows [Semantic Versioning](http://semver.org/) for versioning.
Each service in this repository is versioned on its own.

## Security

Despite everyone’s best efforts, security incidents are inevitable in an increasingly connected world.
OHX is written in Rust to avoid common memory access and memory management pitfalls, even for new contributors.

Industry standards like OAuth and https are used on the external interface level.
Encryption and https certificate management is based on Rustls and Ring (based on boringssl), two very well maintained Rust crates.
No openSSL or C legacy involved.

**On Linux only**: Modern operating system kernel features restrict internet provided executables ("Scripts", "Addons")
via [network user namespaces](https://en.wikipedia.org/wiki/Linux_namespaces#Network_(net)), [cgroups](https://en.wikipedia.org/wiki/Cgroups) and [seccomp](https://en.wikipedia.org/wiki/Seccomp).

### Report a security event or vulnerability

OHX maintainers are committed to provide a secure solution within the limits that are publicly documented.
Should a serious security incident occur, OHX maintainers will:

* Communicate that risk via the websites news blog
* Issue a risk warning message via the cloudconnector Addon (if enabled)

Report any findings to security@openhabx.com.

## Maintainer: Core Deployment

Update the CHANGELOG file before releasing! Use shell scripts for deployment that are found in `scripts/`:

* build.sh: Cross compile for x86_64, armv7l, aarch64 as static musl binaries
* deploy.sh: Deploy to Github Releases as zipped distribution
  and to the Github Package Registry as Docker container, both including the latest revision of the **Setup & Maintenance** Web-UI.

-----
 David Gräff, 2019
