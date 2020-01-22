# OHX Rule Engine

`ohx-ruleengine` is an [Event, Condition, Action](https://en.wikipedia.org/wiki/Event_condition_action) rule engine
with a few flow engine capabilities like passed on state and branches.

"Events" called "Triggers" in this implementation, "Conditions", "Actions" and "Transformations" types are called **Rule Engine Modules**.
The rule engine provides a few build-in modules.

The following document describes the abilities and build-in modules of the engine, but there are no examples
and exact input / output names on purpose. It would be difficult to keep them in-sync with the code.

Instead also check out the examples directory. Those examples are automatically tested before each release.
Use the **Setup & Maintenance** Web UI to create Rules in a graphical fashion.  

## Rule Variables

A global non-persistent variable storage allows to exchange information between different rules and/or different runs.

Modules can accept named **Inputs** and produce named **Outputs**.
Another name would be "Rule Variables". In contrast to global variables those named values only exist during the rule processing.

The "schedule" trigger module for example not only triggers at the configured date/time, but also outputs the current
  and trigger-next date/time via named outputs "now" and "next".
  All preceding conditions, actions, transformations can access those values.
  
## Rule configuration

* A rule can be a singleton (only one rule of that type can run at a time) or run in multiple instances.

## Rule Inputs

A rule contains a section where "Inputs" are defined.
Those can be used as *Named Inputs* for conditions, actions and transformations.

A rule can be easily shared this way and only the input sections requires adaption. 

An "Input" can be 

* a constant json-valid value,
* a global variable reference,
* a Thing property reference,
* a rule status reference,
* a reference to the last message of a notification channel.

## Build-in Rule Engine Modules

#### Transformations

A "transformation" transforms input values (for example an MQTT string value) into something else (for example an OHX Color Value).

* [**math**] Performs the configured math operation ("+", "-", "/", "*", "^" (exponent), "%" (module)) on two named inputs and outputs a "value".
* [**convert**] Convert the given named input by the configured non fallible conversion ("inversion", "negate", "to_string", "round") and outputs a "value".
* [**parse**] Parses a named input value and tries to convert to the configured output type ("number", "rgb color property", "hsv color property"). 
* [**jsonpath**] Extracts a value out of a structured format like json, toml, yaml and outputs a "value".
* [**regex_replace**] Performs a regex replacement for a given named input and outputs a "value" as well as "regex_1", "regex_2" and so on for captured groups.

#### Triggers

* [**schedule**] Triggers on a configured absolute date/time or on a periodic time expressed via a cron expression. 
* [**rules**] Triggers when a configured rule started or stopped.
* [**input_changed**] Triggers when a configured Rule Input (global variable, Thing property) changed.

#### Conditions

* [**is_datetime**] Checks if the current date/time is within the configured range.  
* [**is_in_range**] Checks if the configured named input is within the configured range.  
* [**is_equal**] Checks if the configured named input is equal to the configured value.  
* [**is_running**] Checks if a configured rule is currently running.

#### Actions

* [**notify**] Uses the notification service to publish a notification.
  Named input values can be used for the title and message.
  The default is to publish to all notification service channels, this can be limited if a "channel tag" is used.
  Refer to the notification service documentation of ohx-core.
* [**command**] Issues a command with a given command value to an Addon for a given Thing.
  Commands are Thing specific. If a Thing implements the Switch device trait, you can send a ("switch", "on") command.
* [**script**] Executes the configured script and passes in all named inputs as well as rule inputs.
  The script may set different named outputs. The maximum execution time can be configured, the default is 5 seconds.
  Access to the network / internet can be allowed or disallowed.
* [**rule**] Restart the current rule

## Extendable

The rule engine offers an extension point ("scripts").
Addons and Script engines can register additional Rule Engine Modules.

## Control Flow

A rule contains a set of triggers.
As soon as it is triggered, it will start executing with the first defined Action and proceeds with all following defined Actions.
An action may be accompanied by a set of conditions. Only if those evaluate to true, the action is executed.

An action may have children. Those children are only executed if the conditions of the parent action have been met.
Such a child is again an action accompanied by a set of conditions.

This structure allows to implement branches ()