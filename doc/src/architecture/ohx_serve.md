# OHX Serve

OHX services offer gRPC (remote procedure call) interfaces for interprocess communication.

For easier management a **Setup &amp; Maintenance** web interface is provided as well, which is served via the `ohx-serve` binary.

It also provides an efficient reverse proxy with rate limiting and circuit breaker support which protects against Distributed Denial of Service (DDos) attacks.
An Addon and also core services can register custom http1, https and http/2 endpoints.

The Identity and Access Management (IAM) service for example registers "/token" and "/auth" on https for OAuth2 and the hue emulation IO Service registers the "/api" endpoint on http1 and https.

## Static File Server

A static file server serves all files that are found in the file server root directory (by default `./www-server`), including web-technology based user interfaces.

An Addon container can have a `/web` directory.
That one is mounted in the root directory following the pattern `./addons_http/{addonid}`.
Those files and web applications are accessible via https://openhabx-ip/addons/{addonid}.

## Web-based user interfaces

Web-based user-interfaces like **Setup &amp; Maintenance** are packaged as npm bundles instead of as a container.
The file server service also has an npm downloader integrated to allow for installation of additional web based UIs.
Those files are accessible via https://openhabx-ip/webui/{bundleid} and files are extracted to `./webui/{bundleid}`.

If a `package.json` file is present, no matter if in an addon static directory or npm bundle,
that file will be analysed and checked for the "management" or "control" user interface keywords.

One of the found interfaces will be served as the root https endpoint.
By default this is **Setup &amp; Maintenance**.
Detected interfaces are ordered by a priority system.

**Setup &amp; Maintenance** lists all other interfaces on its home page.
The static file server API and configuration allows to change the default interface and the API also allows you to enumerate over all detected interfaces.
