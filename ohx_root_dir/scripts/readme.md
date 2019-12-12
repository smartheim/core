# Scripts

You need a script engine to be installed.
For example the "scriptengine_quickjs" for `.js` files.
Files are watched for changes and re-loaded if the respective script engine supports this.

Scripts are not a standalone thing.
They are invoked by the rule engine as "condition", "action" or "transformation".
If something can be expressed by rule items only, scripts should be avoided.

Scripts (along with their script engine if necessary) are invoked
as a separate process with restrictions applied.
A script cannot interact with the filesystem or process management or network by default.