# Develop Bindings

This chapter is about developing a Binding for OHX.
Remember that if your service already exposes Web Things via HTTP or follows the MQTT Homie convention, you don't need to write an Addon.

To integrate a service or a device into OHX the framework need to know about supported Things including their configuration, attached properties, events and actions.

{{< img src="/img/doc/addon-binding-thing.svg" >}}

The following sections walk you through defining Things, their properties, events and actions, Thing configuration and how to keep the frameworks knowledge about a Thing and Thing State in sync.

## Programming language

Each supported programming language has a template repository with multiple examples that you can clone, build and play around with.
If you haven't checked one out yet, go back to [Setting up the development enviroment](/developer/addons#setting-up-the-development-enviroment).

Pick the language that you are either familiar with or that helps to solve a problem the easiest way.
For instance, because of the project [openzwave](https://github.com/OpenZWave/open-zwave) that is written in C++, it makes sense to pick the C++ SDK for a ZWave based Addon.

Addons are generally assumed to be event-driven (asynchronous).
The main thread is running an event loop, controlled by `libAddon` which handles various callbacks of the framework.

Examples in this chapter are written in Rust.

## Things

You have learned about the [Things concept](/userguide/addons#things) in the user guide already.
In the developer context, you need to differentiate between a [Thing Description (TD)](https://w3c.github.io/wot-thing-description/) (defined Properties, Actions, Events) and on the other hand a **Thing**, which is an instance of a specific Thing Description.
It might help to imagine a TD of being like a model or template.

If a Thing requires configuration, the TD is also accompanied by a [Configuration Description](/developer/addon#configurations-for-addons).

## Thing Description

A Thing Description (TD) is either declared in code, declaratively in a file or as a combination of both.
It describes the Properties, Actions and Events and has an ID (ascii "a-zA-Z_" string, unique across the binding), a title and description.
Titles and descriptions optionally use BCP47 language codes (eg "en" for English) for translations.
Default keys are omitted (like "readOnly: false", "writeOnly: false").

```yaml
'@context':
  td: https://www.w3.org/2019/wot/td/v1
  iot: https://iot.mozilla.org/schemas
  om: http://www.ontology-of-units-of-measure.org/resource/om-2/
'@type': iot:Lightbulb
actions:
  toggle:
    description: Turn the lamp on or off
events:
  overheating:
    data:
      type: string
    description: Lamp reaches a critical temperature (overheating)
properties:
  status:
    description: current status of the lamp (on|off)
    readOnly: true
    type: string
  temperature:
    description: Lamp temperature
    readOnly: true
    type: number
    minimum: -32.5,
    maximum: 55.2,
    unit: "om:degree_Celsius"
id: lamp_thing
titles:
  en: Lamp Thing
descriptions:
  en: Lamp Thing description
```

The [Thing Description (TD)](https://w3c.github.io/wot-thing-description/) specifies in detail how Events, Actions and Properties are defined and which keys are available.<br>
[Specification](https://w3c.github.io/wot-thing-description/#thing)

The schema is very similar for Events, Actions and Properties.
You define such an item by an ID (like "overheating" in the event case) and optionally provide a title and description or a map of translated titles and descriptions.

*Actions* usually do not carry any additional data.
They are invoked by their ID.<br>[Specification](https://w3c.github.io/wot-thing-description/#actionaffordance)

*Events* optionally have "data", that is of `type` boolean, string, integer, number or array.<br>[Specification](https://w3c.github.io/wot-thing-description/#eventaffordance)

*Properties* can be "readOnly" (default is false) and "writeOnly" (default is false) and must be of `type` boolean, string, integer, number, array, object, binary.

* For the type string you may have a [MIME type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types) set via the "mediaType" key.
* You can restrict valid values of the string type via the "enum" key.
For example: `"enum": ["On", "Off"],`
* You can restrict valid values of the array type via the "items" key.
For example: `"items": ["item1", "item2"],`.
Further restrict the amount of possible selections via "minItems" and "maxItems".
* For the binary type you must have at least one item in the "links" section.
  Each item requires a "href", that points to a relative or absolute position on where to find the binary data.
The HTTP [Content-Type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Type) header must be set for such a binary http endpoint.
* Numbers and integers can be restricted with "minimum" and "maximum".
* All types can be annotated with a "unit".
  You find valid values for units on http://www.ontology-of-units-of-measure.org/resource/om-2/.

To get an idea, here are some more elaborated examples for properties of the string, array and binary type:

{{< code-toggle file="thing_declarative_example1" class="code-toggle-200 mb-4" >}}
properties:
  an_array:
    title: MyArray
    description: A restricted array
    type: array
    items: ["item1", "item2", "item3"]
    minItems: 1
    maxItems: 2
{{< /code-toggle >}}
{{< code-toggle file="thing_declarative_example2" class="code-toggle-200 mb-4" >}}
properties:
  videofeed:
    title: Video
    description: A video feed of a camera
    type: binary
    links:
      - href: /addon/url/to/videofeed.mp4
{{< /code-toggle >}}
{{< code-toggle file="thing_declarative_example3" class="code-toggle-200 mb-4" >}}
properties:
  an_image:
    title: Cat image
    description: Just because
    type: binary
    links:
      - href: /addon/url/to/image.jpg
{{< /code-toggle >}}
{{< code-toggle file="thing_declarative_example4" class="code-toggle-200 mb-4" >}}
properties:
  audio_file_or_stream:
    title: Audio
    description: An interface will not auto-play this audio but render a play button
    type: binary
    links:
      - href: /addon/url/to/audio/stream.m3u
{{< /code-toggle >}}
{{< code-toggle file="thing_declarative_example5" class="code-toggle-200 mb-4" >}}
properties:
  markdown_formatted_text:
    title: Markdown Text
    description: A renderer of this property would format the text if supported
    type: string
    mediaType: "text/markdown"
{{< /code-toggle >}}

### Semantic tagging

{{< img src="/img/doc/brick_schema.png" style="float:left;margin-right:10px" height="50px" maxwidth="50px" >}}

OHX uses the https://iot.mozilla.org/schemas and [Brick schema](https://brickschema.org/) to add some semantic context to Things.
It helps user interfaces to know if a Thing is a light bulb or a rollershutter.
It helps the unit-of-measurement processor to know in which unit your sensor reading is and how to convert it to your region specific unit.
And it helps the natural language processor to know that a temperature Thing is a sensor for when you ask for all sensor readings of a room.

{{< img src="/img/doc/semantic_tags.jpg" maxwidth="100%" >}}

Set the "@type" for Things and Thing properties.
Find available types for Things and Properties here: https://iot.mozilla.org/schemas.

{{< code-toggle file="thing_with_type" class="code-toggle-200" >}}
'@type': VideoCamera
properties:
  videofeed:
    "@type": VideoProperty
{{< /code-toggle >}}


Tag your Thing and properties with "Equipment" tags from the Brick schema.
Tags in that schema follow a hierachy, meaning that tagging a lightbulb with "LED" inherits "Lighting".
Find all tags here: https://github.com/BuildSysUniformMetadata/Brick/blob/master/src/Tags.csv.

{{< code-toggle file="thing_with_type" class="code-toggle-100" >}}
properties:
  temperature:
    tags:
      - HVAC
{{< /code-toggle >}}


### Dynamic TD

The above procedure is very well suited for static Things.
Like a specific light bulb *Thing* where the light bulb device never changes.
You will find yourself using the declarative way most often.

But sometimes your Things are actually not fixed, at least not entirely.
Properties might be added or changed, depending on the capability of a device or service that is mapped to a Thing.

You can also define a TD programmatically, and alter and re-publish a TD at any time.

<div class="mb-2">
<tab-container>
  <tab-header>
    <tab-header-item class="tab-active">Rust</tab-header-item>
  </tab-header>
  <tab-body>
<tab-body-item >{{< md >}}
```rust
use ohx::{ThingDesc, ThingRegistry, Property, Property::PropertyType, Action};
use language_tags::LanguageTag;

fn define_thing(ctx: &AddonContext) {
  // Define actions ...
  let action_refresh = Action::new("refresh")
    //.handler(action_handler) // more on handlers later in this chapter
    .title(langtag!(en), "Refresh Forecast")
    .description(langtag!(en), "Updates the forecast");

  // ...
and properties 
  let property_next12 = Property::new("Next12", PropertyType::Number)
    .type("TemperatureProperty") // optional semantic type, see https://iot.mozilla.org/schemas
    .unit("degree celsius")
    //.handler(property_handler) // a writable property would require a handler
    .title(langtag!(en), "Next 12 hours")
    .description(langtag!(en), "Shows the next 12 hours forecast");

  // ...
and configuration

  // Build the Thing
  let thing = ThingDesc::new("Forecast12hoursPeriod");
  thing.putAction(action_refresh);
  thing.putProperty(property_next12);
  thing.setTitle(langtag!(en), "Forecast for 12 hours")
  thing.setDescription(langtag!(en), "Long description");
  ThingRegistry::publish(ctx, thing);
}

// You might want to read a thing from file (json, yaml) and just alter it,
// using the file like a template
fn define_thing_from_file(ctx: &AddonContext) {
  let thing = ThingDesc::from_file("things/my-thing-id.json").unwrap();
  ThingRegistry::publish(ctx, thing).unwrap();
}

// Read in all files in a directory and publish them
fn define_things_from_file(ctx: &AddonContext) {
  ThingRegistry::publish_files(ctx, "things/").unwrap();
}

fn edit_thing(ctx: &AddonContext) {
  let thing: ThingDesc = ThingRegistry::get(ctx, "Forecast12hoursPeriod").unwrap();
  thing.setTitle(langtag!(en), "Forecast for 12 hours");
  // ...
  thing.putProperty(property_next12);
  ThingRegistry::publish(ctx, thing);
}
```
{{< /md >}}
  </tab-body-item >
  </tab-body>
  </tab-container>
</div>

No matter if declared in code or declaratively specified, `libAddon` will process and push the resulting *Thing Descriptions* to the [Runtime Configuration Storage](/developer/architecture#configuration-storage).

## Thing Instance

Whenever the user accept a *Thing* from the Inbox or creates one manually, for example by creating one in the **Setup &amp; Maintenance** interface,the framework stores a tuple of a unique id (uid), a reference to the thing description (ref), user assigned tags and required configuration

Such a tuple of a Weather forecast Thing (id:`forecast12hour`) of an addon `myweather`, that requires no further configuration would look like this:

```json
{
  "uid": "arbitratry-unique-id",
  "ref": "myweather-forecast12hour",
  "config": {},
  "tags": {}
}
```

Such an object is stored in the [Configuration Database](/developer/architecture#configuration-storage).

Of course it is not enough for just the framework to know about *Things*.
Your addon need to know as well to handle *Thing* specific resources like open network or file handlers and to associate status.
You might want to report back to the user that a *Thing* configuration is invalid.

{{< img src="/img/doc/thing_instance.svg" >}}

That is why you want to model OHX Things as structs or classes within your Addon code.
Those hold the **Thing Data**, as seen above, and your own resources and state.

You instruct the framework to create an object instance of your **Thing class** via an `on_instance_created` handler.

## Handlers &amp; Framework Callbacks

The framework calls you back on various events.
This includes when a thing instance got created, or has been edited (tags or configuration has changed) or when the user expressed his wish to remove a Thing instance.
This is similar to the [Addon Handlers](/developer/addon#addon-registration).

Handlers are stored in the `AddonContext` object and that is where you need to register your handlers for Thing Instances, Thing Actions and Thing Channels.

In the following code snippet an object of the `MyThing` class gets created whenever the user creates a Thing of that specific type.
That happens because a lambda function creates a `MyThing` instance in the `on_instance_created` callback on line 16.

The framework calls `on_instance_created` (find it in the `edit_thing` method) for a particular Thing Data and expects you to return an object that implements the `Thing` interface.
That interface requires a `modified` and `remove` method to be implemented.

<div class="mb-2">
	<tab-container>
		<tab-header>
			<tab-header-item class="tab-active">Rust</tab-header-item>
		</tab-header>
		<tab-body>
<tab-body-item >{{< highlight rust "linenos=table" >}}
use ohx::{AddonContext, ProgressStream, Result, Action, Property, Thing, ThingInstance, ThingData};

struct MyThing {
  data: ThingData;
  private_variables: Option<String>;
}

impl Thing for MyThing {
  fn new(data: ThingData, ctx: &mut AddonContext) -> Option<MyThing>;
  fn modified(&mut self, ctx: &AddonContext, thing: &ThingDesc, data: ThingData) -> Result<()>;
  fn remove(self, ctx: &AddonContext, thing: &ThingDesc) -> Progress;
}

fn edit_thing(ctx: &mut AddonContext) {
  // ...
  ctx.on_instance_created(thing, | data: ThingData, ctx: &mut AddonContext | -> MyThing::new(data, ctx));
  ThingRegistry::publish(ctx, thing);
}
{{< /highlight >}}</tab-body-item >
		</tab-body>
    </tab-container>
</div>


A `ThingData` contains (uid, ref, config, tags) with *uid* being a unique-id string, *ref* being of type `ThingDesc`, config is a json string, and tags a list of strings.

When the user, a rule or an IO Service issues a command, the framework will also call you back.
You would usually register respective Thing property and Thing action callbacks in the constructor / object creation like in the following code snippet.

<div class="mb-2">
	<tab-container>
		<tab-header>
			<tab-header-item class="tab-active">Rust</tab-header-item>
		</tab-header>
		<tab-body>
<tab-body-item >{{< highlight rust "linenos=table" >}}
use ohx::{AddonContext, ProgressStream, Result, Action, Property, Thing, ThingInstanceData};

impl Thing for MyThing {
  fn created(data: ThingData, ctx: &mut AddonContext) -> Option<MyThing> {
    // Create Thing instance
    let &mut instance = MyThing(data: data, private_variables: None);

    // ...
and link actions and property handlers.
AddonContext expects 
    ctx.on_action(data, "my_action",
      |action: &Action| instance.action_handler(self, ctx, action) )

    ctx.on_property_command(data, "brightness",
      |property: &Property| instance.brightness_handler(self, ctx, property) )
    Some(instance)
  }

  fn action_handler(&mut self, ctx: &AddonContext, data: ThingData) -> Progress;
  fn brightness_handler(&mut self, ctx: &AddonContext, data: ThingData) -> Progress;
}

fn edit_thing(ctx: &mut AddonContext) {
  let thing: ThingDesc = ThingRegistry::get(ctx, "Forecast12hoursPeriod").unwrap();
  // ...
  ctx.on_instance_created(thing, | data: ThingData, ctx: &mut AddonContext | -> MyThing::created(data, ctx));
  ThingRegistry::publish(ctx, thing);
}
{{< /highlight >}}</tab-body-item >
		</tab-body>
    </tab-container>
</div>

### Progress Reporting

You might have noticed in the above code snippets, that some function signatures contain a `Progress` instead of a result type.

The framework does not expect you to perform a command or a remove request immediatelly.
Instead it allows you to return a stream of progress events.

Just be aware that some timing restrictions are put on this asynchronous API.
You MUST report progress in periods smaller that 30 seconds and you may not take longer than 5 minutes to fulfill a method call and close the `Progress` stream with a `done` call.

It is important to keep those restrictions in mind, because OHX will forcefully restart your Addon on missbehaviour.

<div class="mb-2">
	<tab-container>
		<tab-header>
			<tab-header-item class="tab-active">Rust</tab-header-item>
		</tab-header>
		<tab-body>
<tab-body-item >{{< highlight rust "linenos=table" >}}
use ohx::{AddonContext, ProgressStream, Result, Action, Property, Thing, ThingInstance, ThingInstanceData};

impl ThingInstance for MyThingInstance {
  fn remove(self, ctx: &AddonContext, thing: &ThingDesc) -> Progress {
    let mut progress = Progress::new("progress_id");

    TidyUp::new().reporter( ||
      // other thread; report progress
      progress.percentage(10);
      // done
      progress.done();
    );

    progress
  }
}
{{< /highlight >}}</tab-body-item >
		</tab-body>
    </tab-container>
</div>

### Populate the Inbox

If you are able to discover devices or services automatically, you would push those to the OHX Inbox.
A user can easily pick them up and has a hassle free experience with your Addon.

A discovery result can have a time-to-live (TTL) value assigned.

<div class="mb-2">
	<tab-container>
		<tab-header>
			<tab-header-item class="tab-active">Rust</tab-header-item>
		</tab-header>
		<tab-body>
<tab-body-item >{{< highlight rust "linenos=table" >}}
use ohx::{AddonContext, ProgressStream, Result, Action, Property, Thing, ThingInstance, ThingInstanceData};

fn discovery_start(ctx: &mut AddonContext) {
  let thing: ThingDesc = ThingRegistry::get(ctx, "Forecast12hoursPeriod").unwrap();
  // ...
  ctx.on_instance_created(thing, | data: ThingData, ctx: &mut AddonContext | -> MyThing::created(data, ctx));
  ThingRegistry::publish(ctx, thing);
}

{{< /highlight >}}</tab-body-item >
		</tab-body>
    </tab-container>
</div>


Notice that you can, if necessary, also push Thing Descriptions (TD) to the framework.
That is required if you need to assemble TDs first, depending on a devices or services discovered capabilities.

Ideally Things that you push to the Inbox are fully configured and usuable as soon as the user has accepted them.

Pairing procedures or only partially discovered Things / Services might require some additional configuration.
What you would do is to present those devices to the frameworks Inbox anyway, but report a "Configuration Required" Thing status.
In a later section we talk about the Thing status.

## Configuration

A Thing might require additional configuration which is described in a *Configuration Description (CD)* JSonSchema file.
This is similar to [Addon Configuration](/developer/addon#configurations-for-addons).
You need to publish CDs to the [Runtime Configuration Storage](/developer/architecture#configuration-storage), so that editors know how to render a user interface for a Thing configuration.

<div class="mb-2">
	<tab-container>
		<tab-header>
			<tab-header-item class="tab-active">Rust</tab-header-item>
		</tab-header>
		<tab-body>
<tab-body-item >{{< highlight rust "linenos=table" >}}
use serde::{Serialize, Deserialize};
use semver::Version;
use ohx::{Config};

// This will be generated by a tool out of the JSonSchema.
// Do not manually specify if possible!
#[derive(Serialize, Deserialize, Debug)]
struct MyThingConfig {
    username: String;
    password: String;
}

fn upgrade_config_id_cb(...) -> Result<String> {
  // ...
handle configuration schema updates, for instance
  // if a field got renamed like from "pwd" to "password.
}

fn main() {
    // ...
    // Publish thing 'thingid' config schemas.
No optional ui schema is given in this example.
    Config::schema_publish(ctx, "thingid", "schema/thing_id_schema.json", None).unwrap();
    // To unpublish, for example when a Thing doesn't exist anymore after an update
    Config::schema_unpublish(ctx, "thingid").unwrap();
}
```
{{< /highlight >}}</tab-body-item >
		</tab-body>
    </tab-container>
</div>

You reference required configuration in your TD:

{{< code-toggle file="thing_config" >}}
'@context':
- https://www.w3.org/2019/wot/td/v1
- https://iot.mozilla.org/schemas
'@type': Lightbulb
configuration:
  - "hue_addon/bridge-config"
  - lightbulbconfig
{{< /code-toggle >}}

"configuration" expects a list.
That is because you can split your configuration into smaller pieces (for easier re-use in multiple Things).
You can also add configuration descriptions of other addons or core addons.
Do this by first name the other addon, append a slash and then the configuration description id like in `hue_addon/bridge-config`.
External addons will not automatically be installed though and user-interfaces are not able to render referenced configuration if such addons are not installed.

### Shared Configuration

You are sometimes in the situation where you like to share configuration between Things.
Think of light bulbs on an IKEA Tradfri or Philips Hue Bridge where all light Things share information on how to access the bridge.

In OHX this is done via **shared Thing configuration**.

Let's think of the Hue bridge example for a moment.
What you would do is:

1.
Create a Hue Bridge Thing which will perform the pairing process via a "pair" action.
2.
Create a configuration description (CD) with the id "bridge-config".
This is how the bridge thing stores the access token.
2.
Your Hue Bulb Things reference the "bridge-config" CD in their "configuration" section.

No matter if you update that shared configuration on any bulb or in the dedicated bridge Thing, it will affect all other bulbs.
So technically, you wouldn't even need a dedicated bridge Thing.
Usually this pattern helps with discovery though, because most often you will first need to configure some form of access, perform a pairing procedure, before you can discover further Things.

### Type safe configuration

Remember that we get something like this by the framework for a **Thing**:

```json
{
  "uid": "arbitratry-unique-id",
  "ref": "myweather-forecast12hour",
  "config": {
    "key": "value",
    "key2": 123,
    "bool": true,
    "complex": {
      "a":"b"
    }
  },
  "tags": {}
}
```

The configuration is accessible as a json string via the object of type `ThingData` that is given to us during object creation and in the `modified` function.

What is not shown in previous sections but would also happen during object creation (for certain languages) is to map the json configuration to a type-safe class object.
If the configuration is invalid, you return an invalid, None or null value back to the framework.

<div class="mb-2">
	<tab-container>
		<tab-header>
			<tab-header-item class="tab-active">Rust</tab-header-item>
		</tab-header>
		<tab-body>
<tab-body-item >{{< highlight rust "linenos=table" >}}
use ohx::{AddonContext, ProgressStream, Result, Action, Property, Thing, ThingInstanceData};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct MyConfig {
  username: String;
}

struct MyThing {
  data: ThingData;
  config: MyConfig;
}

impl Thing for MyThing {
  fn created(data: ThingData, ctx: &mut AddonContext) -> Option<MyThing> {
    // Parse config
    let config: MyConfig = match serde_json::from_str(&data.config) {
      Ok(d) => d,
      Error(e) => {
        ctx.instance_denied("Configuration invalid".to_own());
        return None;
      }
    };

    // Create Thing instance
    let instance = MyThing(data: data, config: config);
    Some(instance)
  }
}

{{< /highlight >}}</tab-body-item >
		</tab-body>
    </tab-container>
</div>

## Addon &amp; Thing State

An Addon and configured Things of a Binding Addon usually have some form of *State*.
Like a light bulb Thing that has a brightness.
This state is expected to be kept / cached within your Addon, so that relative commands like "+5%" can be applied.

OHX expects Addons to synchronize their state to the *State Database*.
That database is queried to display current Addon, Thing and Thing Property States in user interfaces and is used to trigger *Rules*.

Addons are generally not trusted, that's why they cannot connect to the *State Database* directly.
You use `libAddon` instead, which talks to a proxy process ("State Proxy").
That proxy only forwards state updates that match your Addon-ID.

TODO

## Helpers for common protocols

There are a few helper and tool libraries available to ease mapping http endpoints, mqtt topics and coap endpoints to Things and Thing Properties.


The following examples are in Rust.
For the sake of simplicity we assume the same Thing for all three protocols, defined declaratively:

{{< code-toggle file="thing_declarative_example" >}}
'@context':
- https://www.w3.org/2019/wot/td/v1
- https://iot.mozilla.org/schemas
'@type': Lightbulb
actions:
  toggle:
    description: Turn the lamp on or off
events:
  overheating:
    description: Lamp reaches a critical temperature (overheating)
properties:
  on:
    type: boolean
  temperature:
    readOnly: true
    type: string
id: lamp_thing
{{< /code-toggle >}}

### MQTT

We assume an MQTT **state** topic "*light/123/on*" exists for the "on" state, and we send a "false" or "true" command to an MQTT **command** topic "*light/123/on/set*" to switch the lamp on and off.
MQTT only supports strings, so booleans are mapped to "true" and "false" strings implicitely.

We further assume an MQTT state topic on "*light/123/temperature*" and a command topic on "*light/123/toggle"

<div class="mb-2">
<tab-container>
  <tab-header>
    <tab-header-item class="tab-active">Rust</tab-header-item>
  </tab-header>
  <tab-body>
<tab-body-item >{{< md >}}
```rust
use ohx::{Thing, ThingBuilder, ThingRegistry, Property, Property::PropertyType, Action};
use language_tags::LanguageTag;

fn define_thing(ctx: &AddonContext) {
  // Define actions ...
  let action_refresh = Action::new("refresh")
    //.handler(action_handler) // more on handlers later in this chapter
    .title(langtag!(en), "Refresh Forecast")
    .description(langtag!(en), "Updates the forecast");

  // ...
and properties 
  let property_next12 = Property::new("Next12", PropertyType::Number)
    .type("TemperatureProperty") // optional semantic type, see https://iot.mozilla.org/schemas
    .unit("degree celsius")
    //.handler(property_handler) // a writable property would require a handler
    .title(langtag!(en), "Next 12 hours")
    .description(langtag!(en), "Shows the next 12 hours forecast");

  // ...
and configuration

  // Use the Thing builder (builder pattern) and register the Thing
  let thing = ThingDesc::new("Forecast12hoursPeriod", !vec["https://www.w3.org/2019/wot/td/v1", "https://iot.mozilla.org/schemas"]);
  thing.putAction(action_refresh);
  thing.putProperty(property_next12);
  thing.setTitle(langtag!(en), "Forecast for 12 hours")
  thing.setDescription(langtag!(en), "Long description");
  ThingRegistry::publish(ctx, builder.build());
}

fn edit_thing(ctx: &AddonContext) {
  let thing: ThingDesc = ThingRegistry::get(ctx, "Forecast12hoursPeriod").unwrap();
  thing.setTitle(langtag!(en), "Forecast for 12 hours");
  // ...
  thing.putProperty(property_next12);
  ThingRegistry::publish(ctx, thing);
}
```
{{< /md >}}
  </tab-body-item >
  </tab-body>
  </tab-container>
</div>


TODO

### HTTP
TODO

### CoAP

TODO
