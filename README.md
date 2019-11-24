# OHX Core

[![Build Status](https://github.com/openhab-nodes/core/workflows/test/badge.svg)](https://github.com/openhab-nodes/core/actions)
[![](https://img.shields.io/badge/license-MIT-blue.svg)](http://opensource.org/licenses/MIT)

> OHX is a modern smarthome solution, embracing technologies like software containers for language agnostic extensibility. Written in Rust with an extensive test suite, OHX is fast, efficient, secure and fun to work on.

This repository is a [Rust](https://rustup.rs/) workspace. OHX Core consists of multiple services that work in tandem
to form a smarthome solution. Find the individual projects in their respective subdirectories.

For a ready to use operating system image for single board computers and regular PCs (and virtual machines),
you might also be interested in [OHX OS](https://github.com/openhab-nodes/ohx-os/).

## Table of Contents

1. [Download and Start up](#download-and-start-up)
1. [Usage](#usage)
	1. [Command line Options](#command-line-options)
1. [Architecture](#architecture)
1. [Compile and Contribute](#compile-and-contribute)
1. [How to develop Addons](#how-to-develop-addons)
1. [Report a security event or vulnerability](#report-a-security-event-or-vulnerability)
1. [Maintainer: Core Deployment](#maintainer:-core-deployment)

## Download and Start up

**Prerequirement:** OHX uses software containers for Addons.
A software container can easily be restricted in its resource usage (Memory, CPU, File Access, Network Access, limited Kernel API) to protect the host system from potential malicous Addons or just badly written Addons.
You must have [Docker](https://www.docker.com/products/docker-desktop) or a Docker compatible (for example [`podman`](https://podman.io/getting-started/installation)) command line tool installed.

By default `ohx-core` will try to start with http port 80 and https port 443.

> Core is safe to be exposed to the Internet and implements various anti-abuse techniques
like IP rate limiting, Authentication, limited backpressure queues and limited input buffers.

Non-container usage:

1. Download and extract the newest zip file for your hardware.
   Find it on the [releases page](https://github.com/openhab-nodes/core/releases).
2. Your operating system may prevent the use of "system" ports (ports below 1024) though.
   Add the NET_BINDSERVICE capability on Linux / Mac OS like so: `sudo setcap CAP_NET_BIND_SERVICE=+eip ./ohx-core`
2. Start the `ohx-core` binary. 
   - Call `ohx-core -h` to print a list of command line options to adjust OHX's behaviour.

Apart from installing additional addons this method does not require further internet access.

Container usage:

1. Pull the container image of the start binary with
   `docker pull docker.pkg.github.com/openhab-nodes/core/ohx-core`.
2. Start `ohx-core`, which will pull and start up the rest of the images:
   `docker run -rm -p 80:80 -p 443:443 -v ./:/ohx --cap-add NET_BIND_SERVICE -d docker.pkg.github.com/openhab-nodes/core/ohx-core`.
   - Change the `./` part to the directory where you want OHX to store configuration, rules, etc.
   - Use  `docker run -rm -it docker.pkg.github.com/openhab-nodes/core/ohx-core -h` to print a list of command line options to adjust OHX's behaviour.

## Usage

The *Setup & Maintenance* UI can be found on `https://localhost/`, if you are running OHX on the same system,
and on `https://ohx.local/` if you are using [OHX OS](https://github.com/openhab-nodes/ohx-os/).
The pre-configured password for the administrative user is "ohxsmarthome".

You may also use the command line tool (find it in the *releases* zip file) `ohx-cli` to interact with OHX.
Use `ohx-cli --help` to print all available commands and `ohx-cli the_command --help` to show command specific help.

Usually you first want to detect running OHX instances by calling `ohx-cli detect` and than select one of the found
instances to be used for further calls: `ohx-cli login 192.168.1.17:443`.

### Command line Options

The **OHX Root directory** is by default the working directory.
You may change this by starting with `ohx-core -c your_directory`.
By default OHX-Core will create the OHX root directory structure including
`backups`, `certs`, `config`, `scripts`, `things` and `webui`, if it not yet exists.

If you provide an https certificate (x509 in *der* format) via `OHX-ROOT/certs/key.der` and `OHX-ROOT/certs/cert.der`,
OHX Core will use it.

**Ports** are set with `ohx-core -p 80 -s 443` for http and https.

**Container service:** OHX will auto-detect if "docker" or "podman" should be used for container management. "podman" is preferred".
If you want to use "docker" instead, execute with `ohx-core --force-docker`.

OHX is a **device interconnect hub** (ie connect a ZWave wall switch with a Zigbee light bulb) as well as
a smart home implementation via the rule engine.
If you only require the interconnect functionality, start with `ohx-core --no-rule-engine`.

OHX-OS applies a filesystem based **quota for persistent storage** per Addon directory.
It is out of scope for OHX to ensure that limit on a standalone installation accurately.
A best effort approach is used (by watching directories and checking file sizes once in a while),
which simply stops an Addon that exceeds the quota. You can disable this via `ohx-core --disable-quota-enforcement`.

## Architecture

In-depth explanations are given on https://openhabx.com. A quick run down on the architecure follows.

`ohx-core` is a static https file server for web-uis, and a thin supervisior for software containers
(it uses the `docker` or `podman` CLI interface internally) to install, start and manage OHX Addons.
- It generates a self-signed https certificate if none is found at start up and redirects http requests to https.
- Core also routes *Commands* between Addons and between Addons and the *Rule Engine*.
- It acts as a notification and log access service

`ohx-ruleengine` is an [Event, Condition, Action](https://en.wikipedia.org/wiki/Event_condition_action) rule engine.
It uses Yaml files to express rules for easy backups, and rule sharing.
- Addons can register additional "Events", "Conditions", "Actions" and "Transformations".
  A "transformation" transforms an input value (for example an MQTT string value) into something else (for example an OHX Color Value).
- The rule engine provides pre-defined elements. This includes time based "Events" via an internal scheduler, starting /stopping other rules,
  acting on Addon state changes and commands and starting scripts.
- The rule engine is consciously limited but offers an extension point ("scripts").
  Scripts can be used for "Conditions", "Actions" and "Transformations".
  A script will be executed / compiled according to its extension.
  The script engine or the binary will be executed in a very restricted micro container via the "nobody" user, with a non-connected user network namespace, an empty, read-only root filesystem, limited linux API via seccomp and a limited runtime of 5 seconds.
- A rule can be a singleton (only one rule of that type can run at a time) or run in multiple instances.

`ohx-auth` is an Identity and Access Management service, based on OAuth. User accounts are stored in flat files.
It also manages extern OAuth Tokens and token refreshing, like the https://openhabx.com cloud link for
Amazon Alexa and Google Home support.

## Compile and Contribute

OHX is written in [Rust](https://rustup.rs/). 

## How to develop Addons

Please head over to https://openhabx.com to find a step by step guide as well as API overviews.
Find template repositories on https://github.com/openhab-nodes for different programming languages, including Rust, NodeJS, Go and C++.

Recommended Addons in Rust for code inspection and learn by example include:

`hueemulation`: [IO-Service] Registers an http API endpoint on /api and provides the full Hue API, emulating a Hue bridge version 2.
`hue_deconz`: [Binding] Adds support for Zigbee devices via a hue bridge V2 or (deconz) software hue bridge.
   Shows how to use upnp to find bridges.
`mqtt_homie`: [Binding+Transformation] Finds registered MQTT Homie devices on a given MQTT Server.
`mozilla_webthing`: [Binding] Finds Mozilla WebThings in your network. 
`cloudconnector`: [IO-Service] Amazon Alexa and Google Home support via https://openhabx.com account.
   Uses a super fast, lightweight TCP proxy (see [OHX-Cloud](https://github.com/openhab-nodes/cloud) repository) to provide
   a bridge between the Amazon Alexa servers / Google Home Fulfilment Action service and your local OHX installation.
`scriptengine_rust`: Allows scripts to be rust programs. `.rs` script files will be release compiled with the latest Rust compiler.
`scriptengine_nodejs`: Registers nodejs for `.js` javascript files.

You deploy your developed Addon to the OHX Addon Registry via the [OHX-Addon-CLI](https://github.com/openhab-nodes/cloud-addon-registry-cli).

## Report a security event or vulnerability

Despite everyone’s best efforts, security incidents are inevitable in an increasingly connected world.
OHX is written in Rust to avoid common memory access and memory management pitfalls, even for new contributors.

Industry standards like OAuth and https are used on the external interface level.
Modern operating system kernel features restrict internet provided executables ("Scripts", "Addons").

OHX maintainers are committed to provide a secure solution within the limits that are publicly documented.
Should a serious security incident occur, OHX maintainers will:

* Communicate that risk via the websites news blog
* Issue a risk warning message via the cloudconnector Addon (if enabled)

Report any findings to security@openhabx.com.

## Maintainer: Core Deployment

Use shell scripts that are found in `scripts/`:

* build.sh: Cross compile for x86_64, armv7l, aarch64 as static musl binaries
* deploy.sh: Deploy to Github Releases and Github Package Registry (Docker container)

-----
 David Gräff, 2019
