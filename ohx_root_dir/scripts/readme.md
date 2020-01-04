# Scripts

## Script Engines

You need a script engine to be installed.
For example the "scriptengine_quickjs" for `.js` files or the "scriptengine_unix_shell" for `.sh` files.

A file may not be valid for given script engine, please check the Setup & Maintenance UI
or the http API for the last-run/parsing status of a given file.  

## Invocation 
Scripts are invoked by the rule engine as "condition", "action" or "transformation".
If something can be expressed by rule items only, scripts should be avoided.

## Restrictions 

Scripts (along with their script engine if necessary) are invoked
as a separate process with restrictions applied.

A script cannot interact with the filesystem or process management or network by default.

## Caching 
Depending on the script engine, files may be parsed/cached in memory for faster execution.
Such script engines usually watch files for changes and re-load.
