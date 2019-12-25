# Design Principles

OHX is for developers who crave **efficiency**, **speed**, **security**, **maintainability** and **stability** in a home automation system.

#### Efficiency
This design principle often collides with security and maintainability.
And it is indeed true that a secure design comes first.

Often roundtrips, type conversions and additional stress on the operating system in form of fast memory allocations/deallocations can be avoided though.

If a refactoring is required to make a component more efficient, so it be!

#### Speed

In this context it literally means the measureable speed.
Nobody wants to wait 2 seconds for a light to turn on after a switch has been pressed.
 

If efficiency is considered during development, speed is often inherent.


The projects [benchmark suite](/benchmark) is perdiodically used to find regressions of for example rule engine rule invocations per second and http API requests per second.


#### Stability
Stability is a high good for a home control and automation system.

Each individual component is required to implement a selftest and to be prepared to be killed and restarted ("self-healing") at any time.

Strict memory and cpu bounds prevent an unstable overall system if a single component behaves unusually demanding.

Core components must be developed in a programming language that enforces object lifetime awareness.

#### Security
The attack space need to be minimal.
An API Gateway concept exposes only one process to the outside (except additional services with own security concepts).
It acts as a **circuit breaker** and **rate limiter** to help mitigate Distributed Denial of Service (DDOS) attacks.


Each component runs in an isolated container with resource limits, capability and write restrictions in place.
The root filesystem is assumed to be immutable.
<split>
#### Maintainability

There are multiple ways how you can archive this goal.
This project understands a maintainable component as something that has

(1) a well-defined, semantically versioned and documented API,<br>
(2) unit tests,<br>
(3) an integration test.

#### Reuse

A few projects suffer from the Not-Invented-Here syndrom.
OHX strives to use other stable software whenever possible.

Storing state is best done in a [Redis Key-Value Database](https://redis.io) for example.

Storing, compressing and visualising historic state is what [Influx Time Series Database](https://influxdata.com) is best known for.

## Non obligatory

Some design principles are more of a guideline than obligatory.

#### Scaleability

The current architecture is scalable and new services should always be designed in a scalable fashion if possible.

The filesystem write speed (for configuration changes), network interface and memory speed (for configuration descriptions) and REDIS database speed (Thing states) define the limits of the current architecture.
Interprocess communication is implemented via gRPC and allows core services and addons to run distributed.

## Non-goals

#### Backwards compatibility

Providing Long-Term-Stability variants binds developer resources that are rather spend in integrating new solutions and improving existing ones.
OHX follows a fast-phase-out strategy.

Each component provides a versioned API and is able to run next to an older variant.
As soon as all parts of OHX are migrated to a new version, the old component will be decommissioned.

Components are versioned according to [Semantic Versioning](https://semver.org/).

Supporting multiple versions or keep deprecated code is not a goal.
<split>

#### Perfect Code

A contribution is **required** to include unit tests and, if necessary, integration tests.
That way we can automatically make sure that a software piece works under specified use-cases.

OHX components are small in code size and are written in a language (Rust) that prevents wrong memory handling in most cases (crashes).

As long as all tests pass and the API stays intact, even less optimal code is accepted.
It can easily be identified and replaced with better code.
{{< /col3md >}}

## How to develop for OHX

Whenever you want to introduce a new feature, please have a user story in mind (or better, write it down).
For OHX itself for example the user story reads like this:

Early stage:

* John has bought the Philips Hue system.
He also got an Echo and a Xiaomi Vacuum.
* He decides to combine all this with OHX.
* He installs OHX and starts it, according to the instructions linked from the download page.
* He opens the webpage of OHX in his browser (he already knows this procedure because his smart wall sockets required him to enter the wifi credentials via embedded web pages, too)
* He is greeted by a tutorial that explains him some OH concepts, like bindings, things and schedules.
* John doesn't want to read soo much, but thankfully there are many pictures and even an embedded video.
* He knows now that he need to install bindings for Hue, Xiaomi and his Echo.
* His Inbox shows him already his hue bridge and the vacuum.
He adds those.
While adding the things, he is asked to perform one or two pairing processes.
That is something he knows from the Hue and Xiaomi app already.
* Lights pop up in the Inbox.
As expected.
And he adds those that he want to control with OHX.

Side story embedded in full story:

* He can't see his Echo though.

* But the Binding page has the documentation linked and he finds out that there is no auto-discovery, before he has entered his Amazon credentials.
Sounds logical.
* He adds his echo manually via the Thing page.
The Echo Thing is special, because it "consumes" other Things to let the Alexa voice assistant control his home.
He is asked what Things he want to expose to the Echo.
* He now wants to set his hue light on and off to have a first success feeling.
* He is not seeing any "control" part in the interface though, but wait.
He remembers from the tutorial but also from the start/home page that there was another interface very prominently linked.
* He opens that other interface.
It looks awesome, fluid and animated.
He sees his light bulbs on the first page already.
Even sorted by room, because the Hue addon read that information from the Hue bridge during the Inbox Approval.
Astonishing.
He turns the lights on and off and is happy with his selection of OHX because it was so easy to setup.

User stories like that help a lot to streamline interfaces and work flows.
For users as well as programmers that will use your new APIs.

Core services aim to be small in functional coverage.
Because they need to be documented.
Not so much just the API, everyone can read the corresponding code and auto completion does the rest.
But the concepts, architecture and intention of a service or a feature, an API endpoint must be clear.

Cheers, David