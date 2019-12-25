# Walkthrough: A Weather Forecast Binding

<a href="https://www.weather.gov/" style="float:right;max-width:50%" target="_blank" class="card-hover"><img src="/img/doc/usa-national-weather-service.png" class="w-100"></a>

In this chapter we are going to integrate a weather forecast service via the HTTP to Thing utility library.
The entire development process including initial design questions is handled.

We are going to use the National Weather Service (USA), because it does not require any form of authorisation.
Usually you want to register to your favourite, locale weather service and use the API Key in your requests.

### Familiarize with the required HTTP endpoints

To start with, all the endpoints use the API base https://api.weather.gov (as documented on their website). The basic endpoints are all extensions of that original API base to include latitude and longitude values. 

To get familiar with the API, we’re going to get the local weather for Richmond, Va. We’re going to use the following location:

* latitude = 37.540726
* longitude = -77.436050

Let's get the metadata for that location using the metadata endpoint:

    https://api.weather.gov/points/{<latitude>,<longitude>}

So for the Richmond location, this would look like:

    https://api.weather.gov/points/37.540726,-77.436050

A response contains content like this:

```json
{
  "properties": {
     "forecast": "https://api.weather.gov/gridpoints/AKQ/45,76/forecast",
     "forecastHourly": "https://api.weather.gov/gridpoints/AKQ/45,76/forecast/hourly",
     "forecastGridData": "https://api.weather.gov/gridpoints/AKQ/45,76",
  }
}
```

Evaluating the response we find the link for a 12h-period forecast: "https://api.weather.gov/gridpoints/AKQ/45,76/forecast".

A forecast response, again, contains a *properties* key which contains a list of *periods*.
```json
{
  "properties": {
    "periods": [
        {
            "number": 1,
            "name": "Today",
            "startTime": "2019-06-17T11:00:00-04:00",
            "endTime": "2019-06-17T18:00:00-04:00",
            "isDaytime": true,
            "temperature": 93,
            "temperatureUnit": "F",
            "temperatureTrend": null,
            "windSpeed": "6 to 12 mph",
            "windDirection": "SW",
            "icon": "https://api.weather.gov/icons/land/day/sct/tsra_hi,40?size=medium",
            "shortForecast": "Mostly Sunny then Chance Showers And Thunderstorms",
            "detailedForecast": "A chance of showers and thunderstorms between 2pm and 5pm, ..."
        },
    ]
  }
}
```

For all half-structured, in this case json formatted, responses you can use https://app.quicktype.io/ to generate structures / classes for your desired programming language.

### Property topology

We now need to decide on the Thing Property topology.

Always keep in mind that a user-interface need to be able to render your modelled Things.
If your Thing is too complex, user-interfaces will fall back to a mode where they render each Thing Property separately. Chapter [User Interfaces](/developer/frontend_apps) will introduce ways to customize your Thing rendering, at least for the Dashboard App and OHX mobile App. For now we model Things that behave like temperature sensors, so having one primary read-only, number property.

The http API has multiple endpoints for different periods. This should be exposed to the user.
Let's go with two Things called **Forecast12hoursPeriod** for a 12hours period and **Forecast1hourPeriod** for a 1h forecast period.
We then assign properties for today and tomorrow and for now, in 1h, in 2h, in 3h respectively.

    Weather Addon
      -> Thing Forecast12hoursPeriod 
            Properties
            -> Next12 (Number, Unit: °F, Type: temperature, primary)
            -> Today (Number, Unit: °F, Type: temperature)
            -> Tonight (Number, Unit: °F, Type: temperature)
            -> Tomorrow (Number, Unit: °F, Type: temperature)
            -> Tomorrow Night (Number, Unit: °F, Type: temperature)
      -> Thing Forecast1hourPeriod 
            Properties
            -> Now (Number, Unit: °F, Type: temperature, primary)
            -> In1h (Number, Unit: °F, Type: temperature)
            -> In2h (Number, Unit: °F, Type: temperature)
            -> In3h (Number, Unit: °F, Type: temperature)

### Define Things

The structure or definition of a Thing is exposed as [Web Thing Descriptions (TD)](https://w3c.github.io/wot-thing-description/). This can happen either programmatically or by providing the TD file as json or yaml and only connect actions and properties to handler functions afterwards.

#### Programmatic

Let's start with a Rust snippet showing the programmatic way.

```rust
use ohx::{Thing, ThingBuilder, ThingRegistry, Thing::Property::PropertyType, Action};
use language_tags::LanguageTag;

fn refresh_12h_forecast(ctx: &AddonContext, action: &Thing::Action) {
  // Visual feedback for the user
  action.execution_time(10);
  
  // ... do stuff

  // Asynchronous confirmation
  action.confirm();
  // Or on failure. Should be paired with a notification or log entry
  // action.aborted();
}

fn define_thing(ctx: &AddonContext) {
    // Define actions ...
    let action_refresh = Action::new("refresh", &refresh_12h_forecast)
      .title(langtag!(en), "Refresh Forecast")
      .description(langtag!(en), "Updates the forecast");

    // ... and properties 
    let property_next12 = Thing::Property::new("Next12", PropertyType::Number)
      .type("TemperatureProperty") // optional semantic type, see https://iot.mozilla.org/schemas
      .unit("degree celsius")
      //.handler(property_handler) // a writable property would require a handler
      .title(langtag!(en), "Next 12 hours")
      .description(langtag!(en), "Shows the next 12 hours forecast");

    // ... and configuration

    // Use the Thing builder (builder pattern) and register the Thing
    let builder = ThingBuilder::new("Forecast12hoursPeriod", !vec["http://iot.schema.org", "https://iot.mozilla.org/schemas"]);
    builder = builder.addAction(action_refresh)
      .addProperty(property_next12)
      .title(langtag!(en), "Forecast for 12 hours")
      .description(langtag!(en), "Long description");
    ThingRegistry::publish(ctx, builder.build());
}
```

Only one property is shown in the example, but it works the same for all the other properties. A property requires an ID (`Next12`) and a base type (string, number, integer, boolean, object). Optionally you also provide a title and description in as many languages as possible (you might read those from a file). The `type` and `unit` describe the property semantically. In this case we use the https://iot.mozilla.org/schemas/#TemperatureProperty. OHX supports all types of that schema.

TODO 

OHX allows to specify a custom icon for a Thing property. Usually an icon is chosen based on the semantic type of a Thing. For a weather Thing it makes sense to replace the default icon with an actual forecast icon.

You publish your Thing descriptions to a registry. That way user-interfaces and other parts of the framework know about your Things. You can at any time update Thing descriptions and even `unpublish` them.

```rust
use ohx::{Thing, ThingBuilder, ThingRegistry};
use language_tags::LanguageTag;

// No error handling -> don't do that!
fn update_thing(ctx: &AddonContext) {
    // get the Thing
    let mut thing = ThingRegistry::get(ctx, "Forecast12hoursPeriod").unwrap();
    // set a new title
    thing.setTitle(langtag!(en), "new title");
    // change a property
    thing.property_build("Next12").title(langtag!(en), "Next 12 hours").commit();
    // re-publish
    ThingRegistry::publish(ctx, thing);
}
```


Another way to define a property is the declarative way. Put a file with the id of your thing as json or yaml into `things-decl/` in your Addon directory. For example for the **Forecast12hoursPeriod** thing, you would have a file `things-decl/Forecast12hoursPeriod.yml`.

```yaml
id: Forecast12hoursPeriod
title:
  en: Forecast for 12 hours
description:
  en: Long description
properties:
  asd: "a"
actions:
  asd: "a"
events:
  storm_warning: "a"
```

You only need to connect the handlers:

TODO

### Update Thing / Thing Property State

TODO


A property is by default read-only and a http property in particular is by default a GET request with no additional http headers attached. Have a look at the property definition:

```yaml
today:
    context: Temperature
    image:
        uri: https://api.weather.gov/icons/land/day/sct/tsra_hi,40?size=medium
    type: integer
    unit: °F
    http_in:
        cache: 180 # Cache time in minutes
        uri: https://api.weather.gov/gridpoints/AKQ/45,76/forecast
    processors_in:
        - jsonpath:
            path: $.properties.periods[0].temperature # http://jsonpathfinder.com/ helps here
```

We choose an **image** (can be a statically uploaded image or an internet URL), a type, [unit](/developer/addons#unit-of-measurement) and where to get the data from. For an http property that is set via the **http_in** configuration.

As we already know the data is a json encoded object. In OHX we use so called *processors* to transform an input to another value. Via **processors_in** we can define one or multiple [processors](/userguide/thing_connections#processors). We use the *jsonpath* processor to extract the temperature. 

The **context** refers to a specific defined schema, in our case the value represents a "Temperature". This helps user interfaces to render the property correctly.
Schema repositories can be found at http://iotschema.org/ and https://iot.mozilla.org/schemas). The graphical interface will show a selection.
